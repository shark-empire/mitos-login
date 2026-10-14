use mitos_login::security::hash::{hex, hmac_sha256, sha256};
use mitos_login::security::secret::{ct_eq, Secret};
use mitos_login::security::permissions;

#[test]
fn sha256_known_vector_through_the_public_api() {
    assert_eq!(hex(&sha256(b"abc")), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
}

#[test]
fn hmac_sha256_known_vector_through_the_public_api() {
    let mac = hmac_sha256(b"Jefe", b"what do ya want for nothing?");
    assert_eq!(hex(&mac), "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843");
}

#[test]
fn ct_eq_is_a_real_equality_check() {
    assert!(ct_eq(b"same", b"same"));
    assert!(!ct_eq(b"same", b"diff"));
}

#[test]
fn secret_exposes_its_bytes_but_not_via_debug() {
    let s = Secret::from_str("hunter2");
    assert_eq!(s.expose(), b"hunter2");
    let debug_output = format!("{:?}", s);
    assert!(!debug_output.contains("hunter2"), "Debug must never print the secret value");
}

#[test]
fn permissions_reject_a_group_writable_file() {
    let dir = std::env::temp_dir().join(format!("mitos-login-it-perms-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join("f");
    std::fs::write(&p, "x").unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o664)).unwrap();
    assert!(!permissions::is_trusted_root_file(&p));
    std::fs::remove_dir_all(&dir).ok();
}
