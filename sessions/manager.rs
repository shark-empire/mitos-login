//! Tracks the sessions this daemon has launched, keyed by child pid, so the
//! daemon's `SIGCHLD` handler knows what to clean up (close the PAM session,
//! log it, allow the seat to return to the greeter) when a session process
//! exits.
use super::launcher::{self, LaunchOutcome};
use super::selection::SessionType;
use super::session::{Session, SessionState};
use crate::errors::Result;
use crate::sys::pam::PamSession;
use crate::users::User;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

struct Tracked {
    session: Session,
    pam_session: Option<PamSession>,
    #[allow(dead_code)]
    runtime_dir: PathBuf,
}

#[derive(Default)]
pub struct SessionManager {
    by_pid: HashMap<i32, Tracked>,
    next_id: u32,
}

impl SessionManager {
    pub fn new() -> SessionManager {
        SessionManager { by_pid: HashMap::new(), next_id: 1 }
    }

    pub fn has_active_session_on(&self, seat: &str) -> bool {
        self.by_pid.values().any(|t| t.session.seat == seat && t.session.state != SessionState::Ended)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn launch(
        &mut self,
        user: &User,
        session_type: &SessionType,
        pam_session: Option<PamSession>,
        runtime_dir_base: &std::path::Path,
        seat: &str,
        tty: &str,
        lang: &str,
    ) -> Result<u32> {
        let id = self.next_id;
        self.next_id += 1;
        let LaunchOutcome { child, pam_session, runtime_dir } =
            launcher::launch(user, session_type, pam_session, runtime_dir_base, id, seat, tty, lang)?;
        let session = Session {
            id,
            user: user.name.clone(),
            uid: user.uid,
            seat: seat.to_string(),
            pid: child.id() as i32,
            started_at: SystemTime::now(),
            state: SessionState::Running,
        };
        let pid = session.pid;
        self.by_pid.insert(pid, Tracked { session, pam_session, runtime_dir });
        // `child` (a std::process::Child) is dropped here on purpose: its
        // `Drop` impl only closes the stdio handles we set to /dev/null in
        // sys::spawn — it does NOT call waitpid, so this cannot race or
        // interfere with the daemon's own SIGCHLD-driven `waitpid(-1,
        // WNOHANG)` loop, which is what actually reaps the process (see
        // `reap` below). Holding onto `Child` itself would add nothing:
        // everything we still need (pid, session bookkeeping, the PAM
        // handle) is already captured in `Tracked`.
        Ok(id)
    }

    /// Called once for every pid the daemon's `SIGCHLD` handler reaps.
    /// `None` if `pid` was not one of ours (e.g. a grandchild reparented to
    /// us, or noise).
    pub fn reap(&mut self, pid: i32) -> Option<Session> {
        let tracked = self.by_pid.remove(&pid)?;
        launcher::close_after_exit(tracked.pam_session);
        let mut session = tracked.session;
        session.state = SessionState::Ended;
        Some(session)
    }

    pub fn active_sessions(&self) -> Vec<&Session> {
        self.by_pid.values().map(|t| &t.session).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reap_of_unknown_pid_is_none() {
        let mut mgr = SessionManager::new();
        assert!(mgr.reap(999999).is_none());
    }

    #[test]
    fn ids_increase_monotonically() {
        let mgr = SessionManager::new();
        assert_eq!(mgr.next_id, 1);
    }
}
