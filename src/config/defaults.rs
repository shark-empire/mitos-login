//! Named default values, referenced from [`super::Config::default`] and
//! documented here once rather than as bare numbers scattered through the
//! loader.
use std::time::Duration;

pub const MIN_UID: u32 = 1000;
pub const MAX_UID: u32 = 60000;
pub const FREE_ATTEMPTS: u32 = 3;
pub const LOCKOUT_BASE: Duration = Duration::from_secs(5);
pub const LOCKOUT_MAX: Duration = Duration::from_secs(15 * 60);
pub const AUTH_TIMEOUT: Duration = Duration::from_secs(60);
pub const GREETER_IDLE_TIMEOUT: Duration = Duration::from_secs(120);
pub const CONTROL_SOCKET_PATH: &str = "/run/mitos-login/control.sock";
pub const STATE_DIR: &str = "/var/lib/mitos-login";
pub const SESSIONS_DIR: &str = "/usr/share/mitos-sessions";
pub const FRAMEBUFFER_DEVICE: &str = "/dev/fb0";
pub const INPUT_DEVICE_DIR: &str = "/dev/input";
pub const RUNTIME_DIR_BASE: &str = "/run/user";
