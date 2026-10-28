//! Per-user exponential-backoff lockout, persisted to disk so a daemon
//! restart cannot reset an attacker's failure count. The state file is
//! HMAC-SHA256-signed with a machine-local key that never leaves this host;
//! this is a speed bump against a local write-access bug or race, not the
//! primary control (the file lives under `/var/lib/mitos-login`, root:root
//! `0600`, which is the real boundary).
use super::hash::hmac_sha256;
use super::policy::SecurityPolicy;
use super::secret::ct_eq;
use crate::errors::Result;
use std::collections::HashMap;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, Default)]
struct UserState {
    failed_attempts: u32,
    last_failure_unix: u64,
}

pub struct Lockout {
    policy: SecurityPolicy,
    state_path: PathBuf,
    key_path: PathBuf,
    key: Vec<u8>,
    users: HashMap<String, UserState>,
    /// Set once if the on-disk state failed its signature check, so the
    /// daemon can log it and apply a global cool-down for this boot.
    pub tamper_detected: bool,
}

fn now_unix() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}

fn load_or_create_key(path: &Path) -> Result<Vec<u8>> {
    if let Ok(bytes) = std::fs::read(path) {
        if bytes.len() == 32 {
            return Ok(bytes);
        }
    }
    let key = super::urandom(32)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(path)?;
    f.write_all(&key)?;
    Ok(key)
}

impl Lockout {
    pub fn open(state_dir: &Path, policy: SecurityPolicy) -> Result<Lockout> {
        std::fs::create_dir_all(state_dir)?;
        let state_path = state_dir.join("lockout.state");
        let key_path = state_dir.join("lockout.key");
        let key = load_or_create_key(&key_path)?;
        let (users, tamper_detected) = Self::load(&state_path, &key);
        Ok(Lockout { policy, state_path, key_path: key_path.clone(), key, users, tamper_detected })
    }

    fn load(path: &Path, key: &[u8]) -> (HashMap<String, UserState>, bool) {
        let raw = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => return (HashMap::new(), false),
        };
        let mut lines: Vec<&str> = raw.lines().collect();
        let sig_line = match lines.pop() {
            Some(l) => l,
            None => return (HashMap::new(), false),
        };
        let expected_sig = sig_line.strip_prefix("SIG ").unwrap_or("");
        let data = lines.join("\n");
        let mac = hmac_sha256(key, data.as_bytes());
        let mac_hex = super::hash::hex(&mac);
        if !ct_eq(mac_hex.as_bytes(), expected_sig.as_bytes()) {
            return (HashMap::new(), true);
        }
        let mut map = HashMap::new();
        for line in lines {
            let mut parts = line.splitn(3, '\t');
            let (Some(user), Some(count_s), Some(ts_s)) = (parts.next(), parts.next(), parts.next()) else {
                continue;
            };
            let (Ok(count), Ok(ts)) = (count_s.parse::<u32>(), ts_s.parse::<u64>()) else {
                continue;
            };
            map.insert(user.to_string(), UserState { failed_attempts: count, last_failure_unix: ts });
        }
        (map, false)
    }

    fn persist(&self) -> Result<()> {
        let mut data = String::new();
        for (user, st) in self.users.iter() {
            data.push_str(&format!("{}\t{}\t{}\n", user, st.failed_attempts, st.last_failure_unix));
        }
        let data = data.trim_end_matches('\n').to_string();
        let mac = hmac_sha256(&self.key, data.as_bytes());
        let mut out = data;
        out.push('\n');
        out.push_str("SIG ");
        out.push_str(&super::hash::hex(&mac));
        out.push('\n');
        let tmp = self.state_path.with_extension("tmp");
        let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(&tmp)?;
        f.write_all(out.as_bytes())?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, &self.state_path)?;
        Ok(())
    }

    /// `Ok(())` if `user` may attempt authentication right now, `Err` with
    /// how much longer to wait otherwise. Normalizes case the same way
    /// [`crate::users`] does, so `Alice` and `alice` share one counter.
    pub fn check(&self, user: &str) -> std::result::Result<(), Duration> {
        let key = user.to_lowercase();
        let st = match self.users.get(&key) {
            Some(s) => *s,
            None => return Ok(()),
        };
        let wait = self.policy.backoff_for(st.failed_attempts);
        if wait.is_zero() {
            return Ok(());
        }
        let elapsed = now_unix().saturating_sub(st.last_failure_unix);
        let wait_secs = wait.as_secs();
        if elapsed >= wait_secs {
            Ok(())
        } else {
            Err(Duration::from_secs(wait_secs - elapsed))
        }
    }

    pub fn record_failure(&mut self, user: &str) -> Result<()> {
        let key = user.to_lowercase();
        let st = self.users.entry(key).or_default();
        st.failed_attempts = st.failed_attempts.saturating_add(1);
        st.last_failure_unix = now_unix();
        self.persist()
    }

    pub fn record_success(&mut self, user: &str) -> Result<()> {
        let key = user.to_lowercase();
        if self.users.remove(&key).is_some() {
            self.persist()?;
        }
        Ok(())
    }

    /// Re-signs an empty state file after tampering was detected, so the
    /// corrupt copy is not repeatedly reported.
    pub fn reset_after_tamper(&mut self) -> Result<()> {
        if !self.tamper_detected {
            return Ok(());
        }
        self.users.clear();
        self.tamper_detected = false;
        self.persist()
    }

    pub fn key_path(&self) -> &Path {
        &self.key_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("mitos-login-lockout-{}-{}", std::process::id(), name))
    }

    #[test]
    fn allows_then_locks_out_then_recovers_with_time() {
        let dir = tmp("basic");
        std::fs::remove_dir_all(&dir).ok();
        let mut policy = SecurityPolicy::default();
        policy.free_attempts = 2;
        policy.lockout_base = Duration::from_millis(1);
        policy.lockout_max = Duration::from_millis(50);
        let mut lo = Lockout::open(&dir, policy).unwrap();
        assert!(lo.check("bob").is_ok());
        lo.record_failure("bob").unwrap();
        lo.record_failure("bob").unwrap();
        assert!(lo.check("bob").is_ok(), "still within free attempts");
        lo.record_failure("bob").unwrap();
        assert!(lo.check("BOB").is_err(), "must be locked out now, case-insensitively");
        std::thread::sleep(Duration::from_millis(60));
        assert!(lo.check("bob").is_ok(), "must recover after the backoff elapses");
        lo.record_success("bob").unwrap();
        assert!(lo.users.get("bob").is_none());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn persists_across_reopen_and_detects_tampering() {
        let dir = tmp("persist");
        std::fs::remove_dir_all(&dir).ok();
        {
            let mut lo = Lockout::open(&dir, SecurityPolicy::default()).unwrap();
            lo.record_failure("carol").unwrap();
            lo.record_failure("carol").unwrap();
        }
        {
            let lo2 = Lockout::open(&dir, SecurityPolicy::default()).unwrap();
            assert!(!lo2.tamper_detected);
            assert_eq!(lo2.users.get("carol").unwrap().failed_attempts, 2);
        }
        // Tamper with the state file directly, as an attacker with a
        // filesystem bug/race would.
        let state = dir.join("lockout.state");
        let content = std::fs::read_to_string(&state).unwrap();
        let tampered = content.replace("carol\t2", "carol\t0");
        std::fs::write(&state, tampered).unwrap();
        let lo3 = Lockout::open(&dir, SecurityPolicy::default()).unwrap();
        assert!(lo3.tamper_detected, "signature must not validate after tampering");
        assert!(lo3.users.is_empty(), "tampered state must not be trusted at all");
        std::fs::remove_dir_all(&dir).ok();
    }
}
