//! Discovers installable "sessions" the way X/Wayland display managers do:
//! small desktop-entry-style files naming a display name and an `Exec=`
//! command. This is what a real `mitos-session` install (or a fallback
//! terminal, or anything else) registers itself with — mitos-login never
//! hard-codes a sibling project's binary name.
use crate::security::permissions;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionType {
    pub id: String,
    pub name: String,
    pub exec: String,
}

fn parse_session_file(content: &str) -> Option<(String, String)> {
    let mut name = None;
    let mut exec = None;
    for line in content.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("Name=") {
            name = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("Exec=") {
            exec = Some(v.trim().to_string());
        }
    }
    Some((name?, exec?))
}

/// Reads every trusted (root-owned, non-writable) `*.session` file directly
/// under `dir`. A directory that does not exist yields an empty list rather
/// than an error — a fresh install with no sessions registered yet is a
/// normal, recoverable state the greeter should still be able to show
/// something sensible for.
pub fn discover(dir: &Path) -> BTreeMap<String, SessionType> {
    let mut out = BTreeMap::new();
    if !permissions::is_trusted_root_dir(dir) {
        return out;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("session") {
            continue;
        }
        if !permissions::is_trusted_root_file(&path) {
            continue;
        }
        let id = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Some((name, exec)) = parse_session_file(&content) {
                out.insert(id.clone(), SessionType { id, name, exec });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_well_formed_entry() {
        let s = "[Session]\nName=MITOS Desktop\nExec=/usr/bin/mitos-session\n";
        let (name, exec) = parse_session_file(s).unwrap();
        assert_eq!(name, "MITOS Desktop");
        assert_eq!(exec, "/usr/bin/mitos-session");
    }

    #[test]
    fn missing_field_yields_none() {
        assert!(parse_session_file("Name=Only Name\n").is_none());
        assert!(parse_session_file("Exec=/bin/sh\n").is_none());
    }
}
