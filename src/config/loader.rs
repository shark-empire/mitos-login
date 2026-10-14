//! Applies one config file's contents onto an in-progress [`super::Config`].
//! Each of the seven files owns a distinct, non-overlapping set of keys —
//! see docs/architecture.md's configuration section for the full reference.
use super::parser::{self, Document, TOP_LEVEL};
use super::Config;
use crate::errors::Result;
use crate::localization::Language;
use std::path::Path;
use std::time::Duration;

fn read(path: &Path) -> Result<Option<Document>> {
    match std::fs::read_to_string(path) {
        Ok(src) => Ok(Some(parser::parse(&src).map_err(|e| {
            crate::errors::Error::config(format!("{}: {}", path.display(), e))
        })?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn apply_file(cfg: &mut Config, path: &Path) -> Result<()> {
    let Some(doc) = read(path)? else { return Ok(()) };
    match path.file_name().and_then(|n| n.to_str()) {
        Some("login.toml") => apply_login(cfg, &doc),
        Some("greeter.toml") => apply_greeter(cfg, &doc),
        Some("users.toml") => apply_users(cfg, &doc),
        Some("security.toml") => apply_security(cfg, &doc),
        Some("session.toml") => apply_session(cfg, &doc),
        Some("display.toml") => apply_display(cfg, &doc),
        Some("autologin.toml") => apply_autologin(cfg, &doc),
        _ => {}
    }
    Ok(())
}

fn apply_login(cfg: &mut Config, doc: &Document) {
    if let Some(v) = doc.get_str(TOP_LEVEL, "language").and_then(Language::from_code) {
        cfg.language = v;
    }
    if let Some(v) = doc.get_str(TOP_LEVEL, "pam_library") {
        cfg.pam_library = v.to_string();
    }
    if let Some(v) = doc.get_str(TOP_LEVEL, "greeter_user") {
        cfg.greeter_user = v.to_string();
    }
    if let Some(v) = doc.get_str(TOP_LEVEL, "sessions_dir") {
        cfg.sessions_dir = v.into();
    }
    if let Some(v) = doc.get_str(TOP_LEVEL, "state_dir") {
        cfg.state_dir = v.into();
    }
    if let Some(v) = doc.get_str(TOP_LEVEL, "control_socket_path") {
        cfg.control_socket_path = v.into();
    }
    if let Some(v) = doc.get_str(TOP_LEVEL, "runtime_dir_base") {
        cfg.runtime_dir_base = v.into();
    }
}

fn apply_greeter(cfg: &mut Config, doc: &Document) {
    if let Some(v) = doc.get_bool("accessibility", "high_contrast") {
        cfg.accessibility.high_contrast = v;
    }
    if let Some(v) = doc.get_bool("accessibility", "large_text") {
        cfg.accessibility.large_text = v;
    }
    if let Some(v) = doc.get_bool("accessibility", "reduced_motion") {
        cfg.accessibility.reduced_motion = v;
    }
    if let Some(v) = doc.get_bool("accessibility", "screen_reader") {
        cfg.accessibility.screen_reader = v;
    }
    if let Some(v) = doc.get_bool("accessibility", "visible_focus") {
        cfg.accessibility.visible_focus = v;
    }
    if let Some(v) = doc.get_str(TOP_LEVEL, "background_path") {
        cfg.background_path = Some(v.into());
    }
}

fn apply_users(cfg: &mut Config, doc: &Document) {
    if let Some(v) = doc.get_u32("list", "min_uid") {
        cfg.user_list.min_uid = v;
    }
    if let Some(v) = doc.get_u32("list", "max_uid") {
        cfg.user_list.max_uid = v;
    }
    if let Some(v) = doc.get_str_list("list", "denied_shells") {
        cfg.user_list.denied_shells = v;
    }
    if let Some(v) = doc.get_str_list("list", "always_hide") {
        cfg.user_list.always_hide = v;
    }
    if let Some(v) = doc.get_str_list("list", "allow_only") {
        cfg.user_list.allow_only = v;
    }
}

fn apply_security(cfg: &mut Config, doc: &Document) {
    if let Some(v) = doc.get_u32("policy", "free_attempts") {
        cfg.security.free_attempts = v;
    }
    if let Some(v) = doc.get_u64("policy", "lockout_base_secs") {
        cfg.security.lockout_base = Duration::from_secs(v);
    }
    if let Some(v) = doc.get_u64("policy", "lockout_max_secs") {
        cfg.security.lockout_max = Duration::from_secs(v);
    }
    if let Some(v) = doc.get_u64("policy", "auth_timeout_secs") {
        cfg.security.auth_timeout = Duration::from_secs(v);
    }
    if let Some(v) = doc.get_u64("policy", "greeter_idle_timeout_secs") {
        cfg.security.greeter_idle_timeout = Duration::from_secs(v);
    }
    if let Some(v) = doc.get_u32("policy", "max_secret_len") {
        cfg.security.max_secret_len = v as usize;
    }
    if let Some(v) = doc.get_u32("policy", "max_username_len") {
        cfg.security.max_username_len = v as usize;
    }
    if let Some(v) = doc.get_bool("policy", "allow_password") {
        cfg.security.allow_password = v;
    }
    if let Some(v) = doc.get_bool("policy", "allow_pin") {
        cfg.security.allow_pin = v;
    }
    if let Some(v) = doc.get_bool("policy", "restrict_root_to_console") {
        cfg.security.restrict_root_to_console = v;
    }
}

fn apply_session(cfg: &mut Config, doc: &Document) {
    if let Some(v) = doc.get_str(TOP_LEVEL, "root_session") {
        cfg.root_session = Some(v.to_string());
    }
    if let Some(v) = doc.get_str(TOP_LEVEL, "sessions_dir") {
        cfg.sessions_dir = v.into();
    }
}

fn apply_display(cfg: &mut Config, doc: &Document) {
    if let Some(v) = doc.get_str(TOP_LEVEL, "framebuffer_device") {
        cfg.framebuffer_device = v.into();
    }
    if let Some(v) = doc.get_str(TOP_LEVEL, "input_device_dir") {
        cfg.input_device_dir = v.into();
    }
    if let Some(v) = doc.get_u32(TOP_LEVEL, "greeter_vt") {
        cfg.greeter_vt = Some(v);
    }
}

fn apply_autologin(cfg: &mut Config, doc: &Document) {
    if let Some(v) = doc.get_str(TOP_LEVEL, "user") {
        if !v.is_empty() {
            cfg.autologin_user = Some(v.to_string());
        }
    }
    if let Some(v) = doc.get_u64(TOP_LEVEL, "delay_secs") {
        cfg.autologin_delay = Duration::from_secs(v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn security_toml_overrides_defaults() {
        let dir = std::env::temp_dir().join(format!("mitos-login-cfgtest-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("security.toml"), "[policy]\nfree_attempts = 7\nallow_pin = true\n").unwrap();
        let mut cfg = Config::default();
        apply_file(&mut cfg, &dir.join("security.toml")).unwrap();
        assert_eq!(cfg.security.free_attempts, 7);
        assert!(cfg.security.allow_pin);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_file_is_not_an_error() {
        let mut cfg = Config::default();
        apply_file(&mut cfg, Path::new("/does/not/exist/security.toml")).unwrap();
        assert_eq!(cfg.security.free_attempts, super::super::defaults::FREE_ATTEMPTS);
    }

    #[test]
    fn malformed_file_is_a_loud_error() {
        let dir = std::env::temp_dir().join(format!("mitos-login-cfgtest2-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("security.toml"), "this is not valid\n").unwrap();
        let mut cfg = Config::default();
        assert!(apply_file(&mut cfg, &dir.join("security.toml")).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }
}
