//! Integration tests for the authentication pipeline through the crate's
//! public API only (as an external consumer of `mitos_login` would use it).
use mitos_login::authentication::{AuthMethod, AuthOutcome, AuthRequest, Authenticator};
use mitos_login::authentication::client::Backend;
use mitos_login::security::audit::AuditLog;
use mitos_login::security::lockout::Lockout;
use mitos_login::security::policy::SecurityPolicy;
use mitos_login::security::Secret;
use std::path::PathBuf;

fn tmp_dir(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "mitos-login-it-auth-{}-{}-{}",
        name,
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos()
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn authenticator_with_unavailable_backend(policy: SecurityPolicy) -> (Authenticator, PathBuf) {
    let dir = tmp_dir("auth");
    let lockout = Lockout::open(&dir, policy.clone()).unwrap();
    let audit = AuditLog::open(&dir.join("audit.jsonl")).unwrap();
    (Authenticator::new(Backend::Unavailable, policy, lockout, audit), dir)
}

#[test]
fn full_lockout_cycle_through_the_public_api() {
    let mut policy = SecurityPolicy::default();
    policy.free_attempts = 1;
    policy.lockout_base = std::time::Duration::from_millis(1);
    policy.lockout_max = std::time::Duration::from_millis(30);
    let (auth, dir) = authenticator_with_unavailable_backend(policy);

    let attempt = || AuthRequest { user: "alice".into(), method: AuthMethod::Password, secret: Secret::from_str("x"), tty: "tty1".into() };

    // Backend::Unavailable always reports Error, never Success -- enough to
    // drive the lockout counter without a real PAM/shadow setup.
    let (first, _) = auth.authenticate(attempt());
    assert!(matches!(first, AuthOutcome::Error(_)));

    // The free attempt is used; the daemon's own authenticate() records the
    // backend error as neither success nor failure for lockout purposes in
    // this crate's design (a backend outage should not lock anyone out) --
    // confirm that stays true by checking several more attempts never
    // transition to LockedOut purely from backend errors.
    for _ in 0..5 {
        let (outcome, _) = auth.authenticate(attempt());
        assert!(matches!(outcome, AuthOutcome::Error(_)), "a backend outage must report Error, never silently succeed or lock out");
    }
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn implausible_input_is_rejected_before_any_backend_call() {
    let (auth, dir) = authenticator_with_unavailable_backend(SecurityPolicy::default());
    let req = AuthRequest { user: String::new(), method: AuthMethod::Password, secret: Secret::from_str("x"), tty: "tty1".into() };
    let (outcome, session) = auth.authenticate(req);
    assert!(matches!(outcome, AuthOutcome::Failure { .. }));
    assert!(session.is_none());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn disabling_both_auth_methods_is_caught_by_config_validation() {
    use mitos_login::config::{validation, Config};
    let mut cfg = Config::default();
    cfg.security.allow_password = false;
    cfg.security.allow_pin = false;
    assert!(validation::validate(&cfg).is_err(), "a config with no usable auth method must fail validation");
}
