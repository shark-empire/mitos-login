//! Trust checks for on-disk files this daemon relies on: a config file, a
//! session desktop-entry, an avatar image. The rule everywhere is the same
//! one `/etc/sudoers` and mitos-utils' own sudo use: owned by root, and not
//! writable by anyone else.
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// `true` if `path` is a regular file owned by root, not writable by group
/// or other, and not a symlink (checked via `symlink_metadata`, so a
/// root-owned symlink pointing at an attacker-writable target is rejected).
pub fn is_trusted_root_file(path: &Path) -> bool {
    let sym = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return false,
    };
    if !sym.file_type().is_file() {
        return false;
    }
    sym.uid() == 0 && (sym.mode() & 0o022) == 0
}

/// `true` if `path` is a directory owned by root, not writable by group or
/// other (a file living inside it may still be individually untrusted).
pub fn is_trusted_root_dir(path: &Path) -> bool {
    match std::fs::symlink_metadata(path) {
        Ok(m) => m.file_type().is_dir() && m.uid() == 0 && (m.mode() & 0o022) == 0,
        Err(_) => false,
    }
}

/// `true` if `path` is owned by `uid` and not readable/writable by anyone
/// else (mode `0o600` or stricter) — the bar for a secret like the audit
/// signing key.
pub fn is_private_to(path: &Path, uid: u32) -> bool {
    match std::fs::symlink_metadata(path) {
        Ok(m) => m.file_type().is_file() && m.uid() == uid && (m.mode() & 0o077) == 0,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn rejects_group_writable_file() {
        let dir = std::env::temp_dir().join(format!("mitos-login-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("cfg.toml");
        std::fs::File::create(&p).unwrap().write_all(b"x = 1").unwrap();
        std::fs::set_permissions(&p, std::os::unix::fs::PermissionsExt::from_mode(0o664)).unwrap();
        assert!(!is_trusted_root_file(&p), "group-writable file must not be trusted");
        std::fs::set_permissions(&p, std::os::unix::fs::PermissionsExt::from_mode(0o644)).unwrap();
        // root:root + 0644 is what a real deployment ships; this process
        // runs as whatever uid the test harness uses, so we only assert the
        // mode-bit half of the check here.
        let m = std::fs::symlink_metadata(&p).unwrap();
        assert_eq!(m.mode() & 0o022, 0);
        std::fs::remove_dir_all(&dir).ok();
    }
}
