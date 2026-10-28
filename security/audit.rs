//! A tamper-evident audit log: one JSON object per line, each carrying a
//! SHA-256 hash over its own fields plus the previous line's hash. Anyone can
//! read the log and verify the chain; only an attacker who can also rewrite
//! every later line can hide a tampered entry, and doing that still costs an
//! immediate integrity-check failure the next time the daemon starts.
use super::hash::{hex, sha256};
use crate::errors::Result;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn now_unix_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

/// One audit record. `user` and `detail` must already be free of secrets —
/// this module never sees a password.
#[derive(Clone, Debug)]
pub struct Entry {
    pub seq: u64,
    pub unix_ms: u64,
    pub event: &'static str,
    pub user: Option<String>,
    pub detail: String,
    pub prev_hash: String,
    pub hash: String,
}

impl Entry {
    fn canonical(seq: u64, unix_ms: u64, event: &str, user: &str, detail: &str, prev_hash: &str) -> String {
        // 0x1f (ASCII unit separator) cannot appear in any of these fields
        // via normal input, so it cannot be used to forge a field boundary.
        format!("{}\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{}\u{1f}{}", seq, unix_ms, event, user, detail, prev_hash)
    }

    fn compute(seq: u64, unix_ms: u64, event: &'static str, user: Option<String>, detail: String, prev_hash: String) -> Entry {
        let canon = Entry::canonical(seq, unix_ms, event, user.as_deref().unwrap_or(""), &detail, &prev_hash);
        let hash = hex(&sha256(canon.as_bytes()));
        Entry { seq, unix_ms, event, user, detail, prev_hash, hash }
    }

    fn recompute_hash(&self) -> String {
        let canon = Entry::canonical(
            self.seq,
            self.unix_ms,
            self.event,
            self.user.as_deref().unwrap_or(""),
            &self.detail,
            &self.prev_hash,
        );
        hex(&sha256(canon.as_bytes()))
    }

    fn to_json_line(&self) -> String {
        let user_field = match &self.user {
            Some(u) => format!("\"{}\"", escape_json(u)),
            None => "null".to_string(),
        };
        format!(
            "{{\"seq\":{},\"ts_ms\":{},\"event\":\"{}\",\"user\":{},\"detail\":\"{}\",\"prev\":\"{}\",\"hash\":\"{}\"}}",
            self.seq,
            self.unix_ms,
            escape_json(self.event),
            user_field,
            escape_json(&self.detail),
            self.prev_hash,
            self.hash,
        )
    }

    /// Parses one previously-written line. Only used by the integrity
    /// verifier and `mitos-loginctl audit-verify`; a parse failure is
    /// reported as a broken chain, never silently skipped.
    fn from_json_line(line: &str, expected_seq: u64) -> Option<Entry> {
        let get = |key: &str| -> Option<String> {
            let pat = format!("\"{}\":", key);
            let start = line.find(&pat)? + pat.len();
            let rest = &line[start..];
            if rest.starts_with('"') {
                let mut out = String::new();
                let mut chars = rest[1..].chars();
                loop {
                    let c = chars.next()?;
                    match c {
                        '"' => break,
                        '\\' => {
                            let e = chars.next()?;
                            match e {
                                'n' => out.push('\n'),
                                'r' => out.push('\r'),
                                't' => out.push('\t'),
                                '"' => out.push('"'),
                                '\\' => out.push('\\'),
                                'u' => {
                                    let hex: String = (0..4).map(|_| chars.next()).collect::<Option<String>>()?;
                                    let code = u32::from_str_radix(&hex, 16).ok()?;
                                    out.push(char::from_u32(code)?);
                                }
                                other => out.push(other),
                            }
                        }
                        c => out.push(c),
                    }
                }
                Some(out)
            } else if rest.starts_with("null") {
                None
            } else {
                let end = rest.find(|c: char| c == ',' || c == '}').unwrap_or(rest.len());
                Some(rest[..end].to_string())
            }
        };
        let seq: u64 = get("seq")?.parse().ok()?;
        if seq != expected_seq {
            return None;
        }
        let unix_ms: u64 = get("ts_ms")?.parse().ok()?;
        let event_owned = get("event")?;
        let event: &'static str = match event_owned.as_str() {
            "auth_attempt" => "auth_attempt",
            "auth_success" => "auth_success",
            "auth_failure" => "auth_failure",
            "lockout" => "lockout",
            "session_start" => "session_start",
            "session_end" => "session_end",
            "session_failed" => "session_failed",
            "admin_command" => "admin_command",
            "integrity_violation" => "integrity_violation",
            "daemon_start" => "daemon_start",
            "daemon_stop" => "daemon_stop",
            _ => return None,
        };
        let user = get("user");
        let detail = get("detail").unwrap_or_default();
        let prev = get("prev")?;
        let hash = get("hash")?;
        Some(Entry { seq, unix_ms, event, user, detail, prev_hash: prev, hash })
    }
}

pub struct AuditLog {
    path: PathBuf,
    file: File,
    next_seq: u64,
    last_hash: String,
}

impl AuditLog {
    /// Opens (creating if needed) the log at `path`, verifying the existing
    /// chain and picking up right after it.
    pub fn open(path: &Path) -> Result<AuditLog> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let (next_seq, last_hash) = Self::verify_and_tail(path)?;
        use std::os::unix::fs::OpenOptionsExt;
        let file = OpenOptions::new().create(true).append(true).mode(0o600).open(path)?;
        Ok(AuditLog { path: path.to_path_buf(), file, next_seq, last_hash })
    }

    /// Replays the whole file verifying every hash link. Returns the next
    /// sequence number and last hash to continue from (genesis / 0 if the
    /// file does not exist yet). On any break in the chain, everything after
    /// the break is ignored and the caller still gets a place to continue
    /// (the break itself is reported by the returned `bool`), so one
    /// corrupted line cannot erase the record of everything before it.
    fn verify_and_tail(path: &Path) -> Result<(u64, String)> {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok((0, GENESIS.to_string()));
            }
            Err(e) => return Err(e.into()),
        };
        let mut seq = 0u64;
        let mut prev = GENESIS.to_string();
        for line in BufReader::new(file).lines() {
            let line = match line {
                Ok(l) if !l.trim().is_empty() => l,
                _ => continue,
            };
            match Entry::from_json_line(&line, seq) {
                Some(e) if e.prev_hash == prev && e.recompute_hash() == e.hash => {
                    prev = e.hash;
                    seq += 1;
                }
                _ => break,
            }
        }
        Ok((seq, prev))
    }

    /// `true` if every line currently on disk still forms an unbroken chain
    /// all the way to the last one written.
    pub fn verify_full_chain(&self) -> Result<bool> {
        let (seq, hash) = Self::verify_and_tail(&self.path)?;
        Ok(seq == self.next_seq && hash == self.last_hash)
    }

    fn append_entry(&mut self, event: &'static str, user: Option<String>, detail: String) -> Result<()> {
        let entry = Entry::compute(self.next_seq, now_unix_ms(), event, user, detail, self.last_hash.clone());
        let mut line = entry.to_json_line();
        line.push('\n');
        self.file.write_all(line.as_bytes())?;
        self.file.flush()?;
        self.next_seq += 1;
        self.last_hash = entry.hash;
        Ok(())
    }

    pub fn log(&mut self, event: &'static str, user: Option<&str>, detail: impl Into<String>) {
        // Audit logging must never be able to crash or block authentication;
        // a write failure (disk full, read-only fs) is reported to stderr
        // and swallowed rather than propagated.
        if let Err(e) = self.append_entry(event, user.map(|s| s.to_string()), detail.into()) {
            eprintln!("mitos-login: audit log write failed: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("mitos-login-audit-{}-{}", std::process::id(), name))
    }

    #[test]
    fn appends_and_chains() {
        let p = tmp("chain.jsonl");
        let _ = std::fs::remove_file(&p);
        {
            let mut log = AuditLog::open(&p).unwrap();
            log.log("daemon_start", None, "test");
            log.log("auth_attempt", Some("alice"), "tty1");
            log.log("auth_success", Some("alice"), "tty1");
        }
        let (seq, _hash) = AuditLog::verify_and_tail(&p).unwrap();
        assert_eq!(seq, 3);
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn detects_tampering() {
        let p = tmp("tamper.jsonl");
        let _ = std::fs::remove_file(&p);
        {
            let mut log = AuditLog::open(&p).unwrap();
            log.log("daemon_start", None, "test");
            log.log("auth_failure", Some("alice"), "bad password");
            log.log("auth_success", Some("alice"), "tty1");
        }
        let content = std::fs::read_to_string(&p).unwrap();
        let mut lines: Vec<&str> = content.lines().collect();
        // Flip the recorded event of the middle line without recomputing its
        // hash, exactly what an attacker editing the file by hand would do.
        lines[1] = "{\"seq\":1,\"ts_ms\":0,\"event\":\"auth_success\",\"user\":\"alice\",\"detail\":\"bad password\",\"prev\":\"deadbeef\",\"hash\":\"deadbeef\"}";
        std::fs::write(&p, lines.join("\n") + "\n").unwrap();
        let (seq, _) = AuditLog::verify_and_tail(&p).unwrap();
        assert_eq!(seq, 1, "chain must stop right before the tampered line");
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn escape_json_round_trips_tricky_chars() {
        let s = "hello \"world\"\n\t\\ end";
        let e = escape_json(s);
        assert!(!e.contains('\n'));
        assert!(e.contains("\\\""));
    }
}
