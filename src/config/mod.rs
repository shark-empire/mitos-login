//! Configuration: a small hand-rolled TOML-subset parser (zero dependencies
//! — see Cargo.toml), reading the seven files under `config/` into one
//! [`Config`] struct. Every file is optional; a fresh install with none of
//! them still boots with sane defaults, since a login manager that refuses
//! to start over a missing config file is a worse failure mode than one that
//! quietly uses good defaults and lets `mitos-loginctl status` show what it
//! picked.
pub mod defaults;
pub mod loader;
pub mod parser;
pub mod validation;

use crate::accessibility::Settings as AccessibilitySettings;
use crate::localization::Language;
use crate::security::policy::SecurityPolicy;
use crate::users::manager::UserListPolicy;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Config {
    pub security: SecurityPolicy,
    pub user_list: UserListPolicy,
    pub accessibility: AccessibilitySettings,
    pub language: Language,
    pub greeter_vt: Option<u32>,
    pub framebuffer_device: PathBuf,
    pub input_device_dir: PathBuf,
    pub sessions_dir: PathBuf,
    pub runtime_dir_base: PathBuf,
    pub state_dir: PathBuf,
    pub control_socket_path: PathBuf,
    pub pam_library: String,
    pub greeter_user: String,
    pub autologin_user: Option<String>,
    pub autologin_delay: Duration,
    pub root_session: Option<String>,
    pub background_path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            security: SecurityPolicy {
                free_attempts: defaults::FREE_ATTEMPTS,
                lockout_base: defaults::LOCKOUT_BASE,
                lockout_max: defaults::LOCKOUT_MAX,
                auth_timeout: defaults::AUTH_TIMEOUT,
                greeter_idle_timeout: defaults::GREETER_IDLE_TIMEOUT,
                ..SecurityPolicy::default()
            },
            user_list: UserListPolicy {
                min_uid: defaults::MIN_UID,
                max_uid: defaults::MAX_UID,
                ..UserListPolicy::default()
            },
            accessibility: AccessibilitySettings::default(),
            language: Language::EnUs,
            greeter_vt: None,
            framebuffer_device: PathBuf::from(defaults::FRAMEBUFFER_DEVICE),
            input_device_dir: PathBuf::from(defaults::INPUT_DEVICE_DIR),
            sessions_dir: PathBuf::from(defaults::SESSIONS_DIR),
            runtime_dir_base: PathBuf::from(defaults::RUNTIME_DIR_BASE),
            state_dir: PathBuf::from(defaults::STATE_DIR),
            control_socket_path: PathBuf::from(defaults::CONTROL_SOCKET_PATH),
            pam_library: String::new(),
            greeter_user: "mitos-greeter".to_string(),
            autologin_user: None,
            autologin_delay: Duration::from_secs(0),
            root_session: None,
            background_path: None,
        }
    }
}

impl Config {
    /// Loads every recognized file directly under `dir`, applying each on
    /// top of [`Config::default`]. A missing file is not an error; a file
    /// that exists but fails to parse is (loud failure beats a silently
    /// half-applied config).
    pub fn load(dir: &Path) -> crate::errors::Result<Config> {
        let mut cfg = Config::default();
        loader::apply_file(&mut cfg, &dir.join("login.toml"))?;
        loader::apply_file(&mut cfg, &dir.join("security.toml"))?;
        loader::apply_file(&mut cfg, &dir.join("session.toml"))?;
        loader::apply_file(&mut cfg, &dir.join("display.toml"))?;
        loader::apply_file(&mut cfg, &dir.join("users.toml"))?;
        loader::apply_file(&mut cfg, &dir.join("autologin.toml"))?;
        loader::apply_file(&mut cfg, &dir.join("greeter.toml"))?;
        validation::validate(&cfg)?;
        Ok(cfg)
    }
}
