//! Verifies the lockout state survives being closed and reopened (simulating
//! a daemon restart), and that tampering is detected on reopen -- both
//! through the public API only.
use mitos_login::security::lockout::Lockout;
use mitos_login::security::policy::SecurityPolicy;
use std::time::Duration;

fn tmp_dir(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!(
        "mitos-login-it-lockout-{}-{}-{}", name, std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos()
    ));
    std::fs::remove_dir_all(&p).ok();
    p
}

#[test]
fn survives_a_simulated_daemon_restart() {
    let dir = tmp_dir("restart");
    {
        let mut lo = Lockout::open(&dir, SecurityPolicy::default()).unwrap();
        lo.record_failure("alice").unwrap();
        lo.record_failure("alice").unwrap();
        lo.record_failure("alice").unwrap();
        // "Restart": lo is dropped here, nothing more written.
    }
    let lo2 = Lockout::open(&dir, SecurityPolicy::default()).unwrap();
    assert!(!lo2.tamper_detected);
    assert!(lo2.check("alice").is_err(), "3 failures (default free_attempts) must already be locked out after reopening");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn a_hand_edited_state_file_is_detected_as_tampered() {
    let dir = tmp_dir("tamper");
    {
        let mut lo = Lockout::open(&dir, SecurityPolicy::default()).unwrap();
        lo.record_failure("bob").unwrap();
    }
    let state_file = dir.join("lockout.state");
    let mut content = std::fs::read_to_string(&state_file).unwrap();
    content = content.replace("bob\t1", "bob\t99999");
    std::fs::write(&state_file, content).unwrap();

    let lo = Lockout::open(&dir, SecurityPolicy::default()).unwrap();
    assert!(lo.tamper_detected);
    assert!(lo.check("bob").is_ok(), "a tampered file must not be trusted, not even to lock someone out harder");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn different_users_have_independent_counters() {
    let dir = tmp_dir("independent");
    let policy = SecurityPolicy { free_attempts: 1, lockout_base: Duration::from_secs(60), ..SecurityPolicy::default() };
    let mut lo = Lockout::open(&dir, policy).unwrap();
    lo.record_failure("alice").unwrap();
    lo.record_failure("alice").unwrap();
    assert!(lo.check("alice").is_err());
    assert!(lo.check("carol").is_ok(), "carol must be unaffected by alice's lockout");
    std::fs::remove_dir_all(&dir).ok();
}
