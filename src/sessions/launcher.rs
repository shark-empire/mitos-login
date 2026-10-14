//! Turns a successful [`crate::authentication::AuthOutcome::Success`] into a
//! running process. The privilege drop itself lives in [`crate::sys::spawn`];
//! this module is about sequencing (PAM session bracketing, environment,
//! `XDG_RUNTIME_DIR`) around that one security-critical call.
use super::environment::base_environment;
use super::selection::SessionType;
use crate::errors::{Error, Result};
use crate::sys::pam::{PamSession, PAM_SILENT};
use crate::sys::{self, Creds, SpawnSpec};
use crate::users::User;
use std::os::unix::fs::DirBuilderExt;
use std::path::PathBuf;
use std::process::Child;

pub struct LaunchOutcome {
    pub child: Child,
    /// Kept alive until the daemon reaps the child and calls
    /// [`close_after_exit`]; dropping it early would run `pam_end` while the
    /// session is still using credentials PAM established.
    pub pam_session: Option<PamSession>,
    pub runtime_dir: PathBuf,
}

/// Creates (or repairs the ownership/mode of) `base/<uid>`, `0700`, owned by
/// the target user — the same contract `XDG_RUNTIME_DIR` promises everywhere
/// else on the system.
fn ensure_runtime_dir(base: &std::path::Path, uid: u32, gid: u32) -> Result<PathBuf> {
    let dir = base.join(uid.to_string());
    if !dir.exists() {
        std::fs::DirBuilder::new().mode(0o700).create(&dir)?;
    }
    sys::chown_path(&dir, uid, gid)?;
    let mut perms = std::fs::metadata(&dir)?.permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o700);
    std::fs::set_permissions(&dir, perms)?;
    Ok(dir)
}

/// Splits `exec` the same way a POSIX shell would for a simple
/// space-separated command line (no quoting/expansion support — a `.session`
/// file is a trusted, root-owned config, not user input, so this stays
/// deliberately simple rather than embedding a shell parser).
fn split_exec(exec: &str) -> Result<(String, Vec<String>)> {
    let mut parts = exec.split_whitespace();
    let program = parts.next().ok_or_else(|| Error::config("empty Exec= in session file"))?.to_string();
    Ok((program, parts.map(|s| s.to_string()).collect()))
}

#[allow(clippy::too_many_arguments)]
pub fn launch(
    user: &User,
    session_type: &SessionType,
    pam_session: Option<PamSession>,
    runtime_dir_base: &std::path::Path,
    session_id: u32,
    seat: &str,
    tty: &str,
    lang: &str,
) -> Result<LaunchOutcome> {
    // Whether `root` gets a plain shell instead of the graphical desktop
    // stack (config/session.toml's `root_session`) is a *caller* decision:
    // the daemon picks `session_type` before calling this function. By the
    // time we are here, `session_type` is simply what will be launched.
    let runtime_dir = ensure_runtime_dir(runtime_dir_base, user.uid, user.gid)?;

    let mut pam_session = pam_session;
    if let Some(session) = pam_session.as_mut() {
        let rc = session.open_session(PAM_SILENT);
        if rc != crate::sys::pam::PAM_SUCCESS {
            return Err(Error::session(format!("pam_open_session failed ({})", session.strerror(rc))));
        }
    }

    let pam_env = pam_session.as_ref().map(|s| s.env()).unwrap_or_default();
    let env = base_environment(user, session_id, seat, runtime_dir.to_string_lossy().as_ref(), tty, lang, "wayland")
        .extend_without_override(pam_env)
        .build();

    let groups = sys::group_list(&user.name, user.gid)?;
    let (program, args) = split_exec(&session_type.exec)?;

    let mut spec = SpawnSpec::new(&program);
    spec.args = args;
    spec.env = env;
    spec.cwd = user.home.clone();
    spec.creds = Some(Creds { uid: user.uid, gid: user.gid, groups });
    spec.new_session = true;
    spec.umask = 0o022;
    spec.no_new_privs = false; // a desktop session legitimately runs setuid helpers (e.g. its own screen-lock escalation path)
    spec.die_with_parent = false; // the session must outlive a daemon restart

    let child = match sys::spawn(&spec) {
        Ok(c) => c,
        Err(e) => {
            if let Some(session) = pam_session.as_mut() {
                let _ = session.close_session(PAM_SILENT);
            }
            return Err(e);
        }
    };

    Ok(LaunchOutcome { child, pam_session, runtime_dir })
}

/// Call once the daemon has reaped the child (`waitpid`). Closes the PAM
/// session and only then lets `pam_session` drop (`pam_end`).
pub fn close_after_exit(mut outcome_pam_session: Option<PamSession>) {
    if let Some(session) = outcome_pam_session.as_mut() {
        let _ = session.close_session(PAM_SILENT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_exec_handles_args() {
        let (p, a) = split_exec("/usr/bin/mitos-session --foo bar").unwrap();
        assert_eq!(p, "/usr/bin/mitos-session");
        assert_eq!(a, vec!["--foo", "bar"]);
    }

    #[test]
    fn split_exec_rejects_empty() {
        assert!(split_exec("   ").is_err());
    }

    #[test]
    fn ensure_runtime_dir_creates_with_right_mode() {
        use std::os::unix::fs::PermissionsExt;
        let base = std::env::temp_dir().join(format!("mitos-login-rundir-{}", std::process::id()));
        std::fs::remove_dir_all(&base).ok();
        let me = sys::getuid();
        let dir = ensure_runtime_dir(&base, me, me).unwrap();
        let mode = std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o700);
        std::fs::remove_dir_all(&base).ok();
    }
}
