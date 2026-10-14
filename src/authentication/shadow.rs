//! Direct `/etc/shadow` + `crypt(3)` authentication, used only when PAM
//! itself cannot be loaded (see [`super::client`]). This intentionally
//! duplicates none of PAM's module logic (no HISTORY, no complexity rules) —
//! it is a minimal, honest fallback, not a PAM replacement.
use crate::errors::Result;
use crate::sys::Crypt;
use std::io::{BufRead, BufReader};

pub const SECONDS_PER_DAY: u64 = 86400;

#[derive(Clone, Debug, Default)]
pub struct ShadowEntry {
    pub hash: String,
    pub last_change_days: Option<i64>,
    pub min_days: Option<i64>,
    pub max_days: Option<i64>,
    pub warn_days: Option<i64>,
    pub inactive_days: Option<i64>,
    pub expire_day: Option<i64>,
}

impl ShadowEntry {
    /// A hash field that means "this account has no usable password" —
    /// `passwd(5)`'s `!`/`*` lock convention, or a genuinely empty field.
    /// Never treated as "no password required".
    pub fn is_locked_or_passwordless(&self) -> bool {
        self.hash.is_empty() || self.hash.starts_with('!') || self.hash.starts_with('*')
    }

    pub fn is_expired(&self, now_days: i64) -> bool {
        matches!(self.expire_day, Some(e) if e >= 0 && now_days > e)
    }

    pub fn password_expired(&self, now_days: i64) -> bool {
        match (self.last_change_days, self.max_days) {
            (Some(lc), Some(max)) if max >= 0 => now_days > lc + max,
            _ => false,
        }
    }
}

fn parse_field(s: &str) -> Option<i64> {
    if s.is_empty() {
        None
    } else {
        s.parse().ok()
    }
}

/// Looks `user` up in `/etc/shadow`. Requires read access to the file (root
/// only) — a permission error is reported, not silently treated as "no such
/// user", so a misconfigured deployment fails loudly instead of locking
/// everyone out with a confusing "authentication failed".
pub fn lookup(user: &str) -> Result<Option<ShadowEntry>> {
    lookup_in(std::path::Path::new("/etc/shadow"), user)
}

pub fn lookup_in(path: &std::path::Path, user: &str) -> Result<Option<ShadowEntry>> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    for line in BufReader::new(file).lines() {
        let line = line?;
        let mut f = line.split(':');
        let name = match f.next() {
            Some(n) => n,
            None => continue,
        };
        if name != user {
            continue;
        }
        let hash = f.next().unwrap_or("").to_string();
        let last_change_days = f.next().and_then(parse_field);
        let min_days = f.next().and_then(parse_field);
        let max_days = f.next().and_then(parse_field);
        let warn_days = f.next().and_then(parse_field);
        let inactive_days = f.next().and_then(parse_field);
        let expire_day = f.next().and_then(parse_field);
        return Ok(Some(ShadowEntry {
            hash,
            last_change_days,
            min_days,
            max_days,
            warn_days,
            inactive_days,
            expire_day,
        }));
    }
    Ok(None)
}

pub fn now_days_since_epoch() -> i64 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    (secs / SECONDS_PER_DAY) as i64
}

/// Verifies `password` for `user` directly against `/etc/shadow`. Distinct
/// from a "user not found" result so callers can decide how much to reveal.
pub enum ShadowResult {
    Ok,
    WrongPassword,
    NoSuchUser,
    AccountLocked,
    AccountExpired,
}

pub fn verify(crypt: &Crypt, user: &str, password: &[u8]) -> Result<ShadowResult> {
    let entry = match lookup(user)? {
        Some(e) => e,
        None => return Ok(ShadowResult::NoSuchUser),
    };
    if entry.is_locked_or_passwordless() {
        return Ok(ShadowResult::AccountLocked);
    }
    if entry.is_expired(now_days_since_epoch()) {
        return Ok(ShadowResult::AccountExpired);
    }
    if crypt.verify(password, &entry.hash) {
        Ok(ShadowResult::Ok)
    } else {
        Ok(ShadowResult::WrongPassword)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_tmp(content: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("mitos-login-shadow-test-{}-{}", std::process::id(), rand_suffix()));
        std::fs::File::create(&p).unwrap().write_all(content.as_bytes()).unwrap();
        p
    }
    fn rand_suffix() -> u64 {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos() as u64
    }

    #[test]
    fn parses_a_normal_line() {
        let p = write_tmp("alice:$6$abc$def:19700:0:99999:7:::\nbob:*:19000:0:99999:7:::\n");
        let e = lookup_in(&p, "alice").unwrap().unwrap();
        assert_eq!(e.hash, "$6$abc$def");
        assert_eq!(e.last_change_days, Some(19700));
        assert_eq!(e.max_days, Some(99999));
        assert!(!e.is_locked_or_passwordless());
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn locked_and_empty_hashes_are_never_passwordless_success() {
        let p = write_tmp("bob:*:19000:0:99999:7:::\ncarol:!:19000:0:99999:7:::\ndave::19000:0:99999:7:::\n");
        assert!(lookup_in(&p, "bob").unwrap().unwrap().is_locked_or_passwordless());
        assert!(lookup_in(&p, "carol").unwrap().unwrap().is_locked_or_passwordless());
        assert!(lookup_in(&p, "dave").unwrap().unwrap().is_locked_or_passwordless());
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn unknown_user_is_none_not_an_error() {
        let p = write_tmp("alice:$6$x$y:19700:0:99999:7:::\n");
        assert!(lookup_in(&p, "eve").unwrap().is_none());
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn expiry_math() {
        let e = ShadowEntry { hash: "$6$x$y".into(), expire_day: Some(100), ..Default::default() };
        assert!(!e.is_expired(99));
        assert!(!e.is_expired(100));
        assert!(e.is_expired(101));
        let no_expiry = ShadowEntry { hash: "$6$x$y".into(), expire_day: None, ..Default::default() };
        assert!(!no_expiry.is_expired(1_000_000));
    }
}
