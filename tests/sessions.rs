use mitos_login::sessions::environment::base_environment;
use mitos_login::sessions::selection;
use mitos_login::users::User;
use std::os::unix::fs::PermissionsExt;

#[test]
fn base_environment_sets_identity_and_xdg_vars() {
    let user = User { name: "alice".into(), uid: 1000, gid: 1000, display_name: "Alice".into(), home: "/home/alice".into(), shell: "/bin/bash".into() };
    let env = base_environment(&user, 7, "seat0", "/run/user/1000", "tty1", "en-US", "wayland").build();
    let get = |k: &str| env.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());
    assert_eq!(get("HOME"), Some("/home/alice".to_string()));
    assert_eq!(get("USER"), Some("alice".to_string()));
    assert_eq!(get("XDG_SESSION_ID"), Some("7".to_string()));
    assert_eq!(get("XDG_RUNTIME_DIR"), Some("/run/user/1000".to_string()));
    assert_eq!(get("LANG"), Some("en-US".to_string()));
}

#[test]
fn session_discovery_ignores_untrusted_files() {
    let dir = std::env::temp_dir().join(format!("mitos-login-it-sessions-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let trusted = dir.join("mitos.session");
    std::fs::write(&trusted, "Name=MITOS Desktop\nExec=/usr/bin/mitos-session\n").unwrap();
    // World-writable: must be rejected even though it parses fine.
    std::fs::set_permissions(&trusted, std::fs::Permissions::from_mode(0o666)).unwrap();
    let found = selection::discover(&dir);
    assert!(found.is_empty(), "a world-writable .session file must never be trusted");

    std::fs::set_permissions(&trusted, std::fs::Permissions::from_mode(0o644)).unwrap();
    // Whether it's now accepted depends on this sandbox's uid owning the
    // file as root, which may not hold in every test environment -- so this
    // second half only checks that a *wrong-permission* file is excluded,
    // not that a correct one is included (that half is covered by
    // selection::tests::parses_a_well_formed_entry inside the crate).
    std::fs::remove_dir_all(&dir).ok();
}
