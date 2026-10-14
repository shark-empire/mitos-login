//! Locating an optional avatar image. There is no image decoder in this
//! crate (zero dependencies) — the greeter draws a coloured initial bubble
//! whenever no avatar file is found, which the theme already makes look
//! intentional rather than like a missing-image placeholder. When a raw
//! framebuffer-ready image *is* found, the daemon simply hands its path to
//! the greeter; decoding an arbitrary user-supplied PNG/JPEG safely would be
//! by far the largest new attack surface in this whole crate, so instead
//! this only accepts a raw pre-decoded `.rgba` file (`WIDTHxHEIGHT` little
//! square, in the file's own name) written by a trusted account-settings
//! tool, never a general image format straight from the account's home
//! directory.
use crate::security::permissions;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvatarImage {
    pub path: PathBuf,
    pub size: u32,
}

fn parse_size_from_name(name: &str) -> Option<u32> {
    let stem = name.strip_suffix(".rgba")?;
    let (w, h) = stem.rsplit_once('x')?;
    let (w, h) = (w.parse::<u32>().ok()?, h.parse::<u32>().ok()?);
    if w == h && (16..=1024).contains(&w) {
        Some(w)
    } else {
        None
    }
}

/// Looks for `/var/lib/mitos-login/avatars/<user>-<N>x<N>.rgba`, the
/// convention a system settings tool is expected to write into. Rejects
/// anything not owned by root with safe permissions, so a user cannot plant
/// an oversized or crafted file to be read by the daemon as another user.
pub fn lookup(user: &str, base_dir: &Path) -> Option<AvatarImage> {
    if user.is_empty() || user.contains('/') || user.contains("..") {
        return None;
    }
    let dir = base_dir.join("avatars");
    if !permissions::is_trusted_root_dir(&dir) {
        return None;
    }
    let entries = std::fs::read_dir(&dir).ok()?;
    let prefix = format!("{}-", user);
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if let Some(rest) = name.strip_prefix(&prefix) {
            if let Some(size) = parse_size_from_name(rest) {
                let path = dir.join(&*name);
                if permissions::is_trusted_root_file(&path) {
                    let expected_len = (size as u64) * (size as u64) * 4;
                    if std::fs::metadata(&path).map(|m| m.len()) == Ok(expected_len) {
                        return Some(AvatarImage { path, size });
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_size_suffix() {
        assert_eq!(parse_size_from_name("128x128.rgba"), Some(128));
        assert_eq!(parse_size_from_name("64x64.rgba"), Some(64));
    }

    #[test]
    fn rejects_mismatched_or_implausible_sizes() {
        assert_eq!(parse_size_from_name("128x64.rgba"), None, "must be square");
        assert_eq!(parse_size_from_name("4x4.rgba"), None, "too small");
        assert_eq!(parse_size_from_name("5000x5000.rgba"), None, "too large");
        assert_eq!(parse_size_from_name("128x128.png"), None, "wrong extension");
    }

    #[test]
    fn rejects_path_traversal_in_username() {
        let dir = std::env::temp_dir();
        assert!(lookup("../../etc", &dir).is_none());
        assert!(lookup("a/b", &dir).is_none());
    }
}
