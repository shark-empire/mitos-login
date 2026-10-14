//! Sanity checks across the fully-merged [`super::Config`] — the kind of
//! mistake a single file's own parsing can't catch (an admin setting
//! `min_uid` above `max_uid` across two different files, an empty session
//! directory path). Fails loudly rather than starting the daemon with a
//! configuration that would misbehave in a hard-to-diagnose way.
use super::Config;
use crate::errors::{Error, Result};

pub fn validate(cfg: &Config) -> Result<()> {
    if cfg.user_list.min_uid > cfg.user_list.max_uid {
        return Err(Error::config("users.toml: min_uid is greater than max_uid"));
    }
    if cfg.security.lockout_base > cfg.security.lockout_max {
        return Err(Error::config("security.toml: lockout_base_secs is greater than lockout_max_secs"));
    }
    if cfg.security.max_secret_len == 0 {
        return Err(Error::config("security.toml: max_secret_len must be at least 1"));
    }
    if cfg.security.max_username_len == 0 {
        return Err(Error::config("security.toml: max_username_len must be at least 1"));
    }
    if !cfg.security.allow_password && !cfg.security.allow_pin {
        return Err(Error::config("security.toml: at least one of allow_password/allow_pin must be true"));
    }
    if let Some(vt) = cfg.greeter_vt {
        if vt == 0 {
            return Err(Error::config("display.toml: greeter_vt must be 1 or greater"));
        }
    }
    for (label, path) in [
        ("login.toml: sessions_dir", &cfg.sessions_dir),
        ("login.toml: state_dir", &cfg.state_dir),
        ("login.toml: runtime_dir_base", &cfg.runtime_dir_base),
        ("login.toml: control_socket_path", &cfg.control_socket_path),
        ("display.toml: framebuffer_device", &cfg.framebuffer_device),
        ("display.toml: input_device_dir", &cfg.input_device_dir),
    ] {
        if !path.is_absolute() {
            return Err(Error::config(format!("{} must be an absolute path, got {}", label, path.display())));
        }
    }
    if let Some(user) = &cfg.autologin_user {
        if user == "root" {
            return Err(Error::config("autologin.toml: autologin as root is not permitted"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        assert!(validate(&Config::default()).is_ok());
    }

    #[test]
    fn catches_inverted_uid_range() {
        let mut cfg = Config::default();
        cfg.user_list.min_uid = 5000;
        cfg.user_list.max_uid = 1000;
        assert!(validate(&cfg).is_err());
    }

    #[test]
    fn catches_both_auth_methods_disabled() {
        let mut cfg = Config::default();
        cfg.security.allow_password = false;
        cfg.security.allow_pin = false;
        assert!(validate(&cfg).is_err());
    }

    #[test]
    fn catches_root_autologin() {
        let mut cfg = Config::default();
        cfg.autologin_user = Some("root".to_string());
        assert!(validate(&cfg).is_err());
    }

    #[test]
    fn catches_relative_path() {
        let mut cfg = Config::default();
        cfg.state_dir = "relative/path".into();
        assert!(validate(&cfg).is_err());
    }
}
