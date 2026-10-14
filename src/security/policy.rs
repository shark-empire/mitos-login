//! Central, config-derived security policy. Every daemon module reads its
//! limits from here instead of hard-coding a number, so `config/security.toml`
//! is the single place an administrator changes them.
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct SecurityPolicy {
    /// Failed attempts allowed before backoff starts.
    pub free_attempts: u32,
    /// Backoff after the free attempts are used: `base * 2^(n - free)`.
    pub lockout_base: Duration,
    pub lockout_max: Duration,
    /// A PAM conversation (typing a password) may not run longer than this.
    pub auth_timeout: Duration,
    /// The greeter returns to the lock/idle screen after this much
    /// inactivity on the login form itself (not the whole desktop).
    pub greeter_idle_timeout: Duration,
    /// Defense in depth: a password/PIN longer than this is rejected before
    /// it ever reaches PAM, so a malformed or hostile greeter cannot hand the
    /// daemon an unbounded buffer.
    pub max_secret_len: usize,
    /// Defense in depth: a typed username longer than this is rejected
    /// before any lookup.
    pub max_username_len: usize,
    pub allow_password: bool,
    pub allow_pin: bool,
    /// Root may only log in on a console seat, never remotely — mirrors
    /// `/etc/securetty` in spirit without depending on that file's format.
    pub restrict_root_to_console: bool,
}

impl Default for SecurityPolicy {
    fn default() -> SecurityPolicy {
        SecurityPolicy {
            free_attempts: 3,
            lockout_base: Duration::from_secs(5),
            lockout_max: Duration::from_secs(15 * 60),
            auth_timeout: Duration::from_secs(60),
            greeter_idle_timeout: Duration::from_secs(120),
            max_secret_len: 1024,
            max_username_len: 256,
            allow_password: true,
            allow_pin: false,
            restrict_root_to_console: true,
        }
    }
}

impl SecurityPolicy {
    /// Backoff for the `n`-th failed attempt (1-based). `0` while still
    /// within the free budget.
    pub fn backoff_for(&self, failed_attempts: u32) -> Duration {
        if failed_attempts <= self.free_attempts {
            return Duration::ZERO;
        }
        let exp = (failed_attempts - self.free_attempts).min(20);
        let scaled = self.lockout_base.as_secs_f64() * 2f64.powi(exp as i32 - 1);
        let secs = scaled.min(self.lockout_max.as_secs_f64());
        Duration::from_secs_f64(secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_backoff_within_free_budget() {
        let p = SecurityPolicy::default();
        assert_eq!(p.backoff_for(0), Duration::ZERO);
        assert_eq!(p.backoff_for(3), Duration::ZERO);
    }

    #[test]
    fn backoff_grows_then_caps() {
        let p = SecurityPolicy::default();
        let a = p.backoff_for(4);
        let b = p.backoff_for(5);
        let c = p.backoff_for(6);
        assert!(a < b && b < c, "backoff must be strictly increasing early on");
        assert_eq!(p.backoff_for(60), p.lockout_max, "must cap at lockout_max");
    }
}
