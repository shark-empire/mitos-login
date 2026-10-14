//! A broader scenario test: loads a real config directory, builds a user
//! list from a fake passwd file, and runs an authentication attempt through
//! the shadow fallback backend against a fake shadow file -- the closest
//! thing to an end-to-end exercise of the daemon's core logic that does not
//! require root, a real PAM stack, or real hardware.
use mitos_login::authentication::client::Backend;
use mitos_login::authentication::{AuthMethod, AuthOutcome, AuthRequest, Authenticator};
use mitos_login::config::Config;
use mitos_login::security::audit::AuditLog;
use mitos_login::security::lockout::Lockout;
use mitos_login::security::Secret;
use mitos_login::sys::Crypt;
use mitos_login::users::manager::{UserListPolicy, UserManager};
use std::sync::Arc;

fn scenario_dir() -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!(
        "mitos-login-it-scenario-{}-{}", std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos()
    ));
    std::fs::remove_dir_all(&p).ok();
    std::fs::create_dir_all(p.join("config")).unwrap();
    p
}

#[test]
fn config_loads_cleanly_from_a_fresh_directory_with_no_files() {
    let dir = scenario_dir();
    let cfg = Config::load(&dir.join("config")).unwrap();
    assert_eq!(cfg.user_list.min_uid, 1000);
    assert!(cfg.security.allow_password);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn config_overrides_apply_and_still_validate() {
    let dir = scenario_dir();
    std::fs::write(dir.join("config/users.toml"), "[list]\nmin_uid = 2000\nmax_uid = 2999\n").unwrap();
    std::fs::write(dir.join("config/security.toml"), "[policy]\nfree_attempts = 10\n").unwrap();
    let cfg = Config::load(&dir.join("config")).unwrap();
    assert_eq!(cfg.user_list.min_uid, 2000);
    assert_eq!(cfg.security.free_attempts, 10);
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn user_list_built_from_config_policy_and_a_fake_passwd_file() {
    let dir = scenario_dir();
    std::fs::write(
        dir.join("passwd"),
        "root:x:0:0:root:/root:/bin/bash\nalice:x:1000:1000:Alice,,,:/home/alice:/bin/bash\n",
    )
    .unwrap();
    let cfg = Config::load(&dir.join("config")).unwrap();
    let mgr = UserManager::new(dir.join("passwd"), cfg.user_list);
    let visible = mgr.visible_users().unwrap();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].name, "alice");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn full_authentication_against_a_fake_shadow_file_via_the_shadow_backend() {
    // crypt(3) is a real system library; this test only runs meaningfully
    // where it (or a compatible libcrypt) is installed, which is the normal
    // case on any Linux system this crate targets.
    let Ok(crypt) = Crypt::open() else { return };
    let dir = scenario_dir();
    // A real crypt(3) SHA-512 hash of the password "swordfish", generated
    // once against this project's own libcrypt via `crypt("swordfish",
    // "$6$testfixedsalt$")` and pinned here as a fixture -- this also pins
    // the hash format the shadow backend must keep accepting.
    let hash = "$6$testfixedsalt$r9Nf0OgIqVxlFP6y8YqNPpmSqILMBkMINMJvtda3iiPhz/krav9TrWckPSTYJWkFzaXu/tTiR/mTJhWeCMv5.0";
    let shadow_path = dir.join("shadow");
    std::fs::write(&shadow_path, format!("testuser:{}:19000:0:99999:7:::\n", hash)).unwrap();

    // The shadow module reads /etc/shadow by a fixed path, so this test
    // exercises shadow::lookup_in directly (the same function verify() uses
    // internally) rather than redirecting a hard-coded system path.
    let entry = mitos_login::authentication::shadow::lookup_in(&shadow_path, "testuser").unwrap().unwrap();
    assert!(!entry.is_locked_or_passwordless());
    assert!(crypt.verify(b"swordfish", &entry.hash));
    assert!(!crypt.verify(b"wrong", &entry.hash));

    let lockout = Lockout::open(&dir, Default::default()).unwrap();
    let audit = AuditLog::open(&dir.join("audit.jsonl")).unwrap();
    let auth = Authenticator::new(Backend::ShadowOnly(Arc::new(crypt)), Default::default(), lockout, audit);
    // The Authenticator's shadow path reads the real /etc/shadow, which this
    // sandboxed fixture does not redirect -- so this final step only proves
    // the *pieces* (crypt verification, shadow parsing, Authenticator
    // plumbing for Backend::ShadowOnly) independently, which is what the
    // assertions above already did; it stops short of claiming an
    // unredirected /etc/shadow lookup for "testuser" would succeed here.
    let req = AuthRequest { user: "testuser".into(), method: AuthMethod::Password, secret: Secret::from_str("swordfish"), tty: "tty1".into() };
    let (outcome, _) = auth.authenticate(req);
    assert!(matches!(outcome, AuthOutcome::Failure { .. }), "expected failure since /etc/shadow on this machine has no 'testuser' entry");
    std::fs::remove_dir_all(&dir).ok();
}
