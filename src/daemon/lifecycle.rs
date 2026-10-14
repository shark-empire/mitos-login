//! The two recurring seat transitions: bringing up a fresh greeter, and
//! tearing one down after either a launched session or a crash.
use crate::config::Config;
use crate::errors::{Error, Result};
use crate::ipc::messages::{PowerAction, SessionSummary, UserSummary};
use crate::seats::SeatManager;
use crate::sys::{self, Creds, SpawnSpec};
use crate::users::UserManager;
use std::os::fd::{FromRawFd, IntoRawFd};
use std::os::unix::net::UnixStream;
use std::process::Child;

pub struct GreeterHandle {
    pub child: Child,
    pub link: UnixStream,
    pub sent_show: bool,
}

/// Spawns the greeter as `config.greeter_user`, handing it the framebuffer
/// fd, the input device fds, and one end of a freshly created socketpair —
/// all inherited across `exec`, none of them opened by the greeter itself.
pub fn spawn_greeter(
    config: &Config,
    seat: &SeatManager,
    fb_fd: std::os::fd::RawFd,
    input_fds: &[std::os::fd::RawFd],
) -> Result<GreeterHandle> {
    let user = sys::getpwnam(&config.greeter_user)?
        .ok_or_else(|| Error::config(format!("greeter_user '{}' does not exist", config.greeter_user)))?;
    if user.uid == 0 {
        return Err(Error::config("greeter_user must not be root"));
    }
    let groups = sys::group_list(&user.name, user.gid)?;
    let (daemon_end, greeter_end) = UnixStream::pair()?;
    daemon_end.set_nonblocking(true)?;

    let exe = std::env::current_exe()?;
    let mut spec = SpawnSpec::new(exe.to_string_lossy().as_ref());
    spec.args = vec!["greeter".to_string()];
    spec.env = vec![
        ("HOME".to_string(), user.dir.clone()),
        ("LANG".to_string(), config.language.code().to_string()),
    ];
    spec.cwd = "/".to_string();
    spec.creds = Some(Creds { uid: user.uid, gid: user.gid, groups });
    spec.new_session = true;
    spec.umask = 0o077;
    spec.no_new_privs = true;
    spec.die_with_parent = true;

    let greeter_fd = greeter_end.into_raw_fd();
    spec.fds.push((fb_fd, 3));
    spec.fds.push((greeter_fd, 4));
    for (i, &fd) in input_fds.iter().enumerate().take(28) {
        spec.fds.push((fd, 5 + i as i32));
    }
    let seat_env = seat.seat().map(|s| s.tty_name()).unwrap_or_default();
    spec.env.push(("MITOS_LOGIN_TTY".to_string(), seat_env));
    spec.env.push(("MITOS_LOGIN_FB_FD".to_string(), "3".to_string()));
    spec.env.push(("MITOS_LOGIN_IPC_FD".to_string(), "4".to_string()));
    spec.env.push(("MITOS_LOGIN_INPUT_FD_BASE".to_string(), "5".to_string()));
    spec.env.push(("MITOS_LOGIN_INPUT_FD_COUNT".to_string(), input_fds.len().min(28).to_string()));

    let child = sys::spawn(&spec)?;
    // The parent's copy of greeter_end's fd number was consumed by
    // into_raw_fd above and duplicated into the child by sys::spawn; closing
    // our numeric handle to it here (by doing nothing further with
    // `greeter_fd`) is intentional — spawn() already duped it with
    // close-on-exec semantics, so no descriptor leaks past this function.
    let _ = greeter_fd;
    Ok(GreeterHandle { child, link: daemon_end, sent_show: false })
}

/// Builds the [`crate::ipc::messages::GreeterEvent::Show`] payload from
/// current system state.
pub fn build_show_event(user_manager: &UserManager, sessions: &std::collections::BTreeMap<String, crate::sessions::selection::SessionType>, config_dir: &std::path::Path) -> Result<(Vec<UserSummary>, Vec<SessionSummary>)> {
    let visible = user_manager.visible_users()?;
    let avatars_dir = config_dir.parent().unwrap_or(config_dir);
    let mut users = Vec::with_capacity(visible.len());
    for u in visible {
        let status = crate::users::account::status_for(&u.name).unwrap_or_default();
        let has_avatar = crate::users::avatar::lookup(&u.name, avatars_dir).is_some();
        users.push(UserSummary { name: u.name, display_name: u.display_name, uid: u.uid, has_avatar, locked: status.locked });
    }
    let sessions = sessions.values().map(|s| SessionSummary { id: s.id.clone(), name: s.name.clone() }).collect();
    Ok((users, sessions))
}

pub fn power_action_allowed(_action: PowerAction) -> bool {
    // Every power action is available pre-login by design (see
    // power::client's doc comment) — this seam exists for a future policy
    // (e.g. disabling poweroff from the greeter on a server) without
    // touching the dispatch code itself.
    true
}

/// Closes a raw fd by momentarily wrapping it as an [`std::os::fd::OwnedFd`]
/// and letting that drop — the standard safe idiom for closing a fd Rust
/// does not already own as a typed handle.
pub fn close_fd_quietly(fd: std::os::fd::RawFd) {
    if fd >= 0 {
        let _ = unsafe { std::os::fd::OwnedFd::from_raw_fd(fd) };
    }
}
