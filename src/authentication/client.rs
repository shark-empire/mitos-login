//! Ties PAM, the shadow fallback, [`crate::security::lockout`] and
//! [`crate::security::audit`] into the one entry point the daemon calls for
//! every login attempt: [`Authenticator::authenticate`].
use super::methods::AuthMethod;
use super::request::AuthRequest;
use super::response::AuthOutcome;
use super::shadow::{self, ShadowResult};
use crate::security::audit::AuditLog;
use crate::security::lockout::Lockout;
use crate::security::policy::SecurityPolicy;
use crate::sys::pam::{PamApi, PamConv, PamSession, PAM_DISALLOW_NULL_AUTHTOK, PAM_ESTABLISH_CRED, PAM_SILENT, PAM_SUCCESS, PAM_TTY};
use crate::sys::Crypt;
use crate::security::Secret;
use std::cell::RefCell;
use std::sync::Arc;

/// Feeds exactly one already-collected secret to PAM's conversation, then
/// refuses any further prompt — a login daemon asks its own question up
/// front and does not let a PAM module improvise additional ones.
struct SingleShotConv {
    secret: Option<Secret>,
}

impl PamConv for SingleShotConv {
    fn prompt(&mut self, _echo: bool, _text: &str) -> Option<Secret> {
        self.secret.take()
    }
    fn info(&mut self, _text: &str) {}
    fn error(&mut self, _text: &str) {}
}

pub enum Backend {
    Pam(Arc<PamApi>),
    ShadowOnly(Arc<Crypt>),
    Unavailable,
}

/// Deliberately single-threaded, matching the rest of mitosOS's daemons: this
/// type is used from one event loop, never shared across threads (`RefCell`
/// enforces that at compile time rather than risking a poisoned `Mutex`).
pub struct Authenticator {
    backend: Backend,
    policy: SecurityPolicy,
    lockout: RefCell<Lockout>,
    audit: RefCell<AuditLog>,
}

fn sanitize_user_for_log(user: &str) -> String {
    user.chars().filter(|c| c.is_ascii_graphic()).take(64).collect()
}

impl Authenticator {
    pub fn new(backend: Backend, policy: SecurityPolicy, lockout: Lockout, audit: AuditLog) -> Authenticator {
        Authenticator { backend, policy, lockout: RefCell::new(lockout), audit: RefCell::new(audit) }
    }

    fn log(&self, event: &'static str, user: Option<&str>, detail: impl Into<String>) {
        self.audit.borrow_mut().log(event, user, detail);
    }

    /// Lets the rest of the daemon (session lifecycle, admin commands,
    /// shutdown) write to the same tamper-evident log authentication events
    /// already use, rather than each maintaining a separate file.
    pub fn log_event(&self, event: &'static str, user: Option<&str>, detail: impl Into<String>) {
        self.log(event, user, detail);
    }

    pub fn verify_audit_log(&self) -> crate::errors::Result<bool> {
        self.audit.borrow().verify_full_chain()
    }

    /// Runs one authentication attempt end to end: input-length guardrails,
    /// lockout check, the actual credential check, then recording the
    /// outcome. Never panics on attacker-controlled input.
    ///
    /// On `AuthOutcome::Success` with the PAM backend, the second element is
    /// the *live* `PamSession` — already past `pam_authenticate`,
    /// `pam_acct_mgmt` and `pam_setcred`, deliberately **not** dropped here.
    /// The caller must pass it to [`crate::sessions::launcher`] to call
    /// `pam_open_session` and, later, `pam_close_session`, before the
    /// session's own `pam_end` runs. Dropping it without doing that would
    /// tear down credentials PAM modules just established (e.g. resource
    /// limits, supplementary groups from `pam_group`) before the session
    /// ever starts.
    pub fn authenticate(&self, req: AuthRequest) -> (AuthOutcome, Option<PamSession>) {
        let user_log = sanitize_user_for_log(&req.user);
        if req.user.is_empty() || req.user.len() > self.policy.max_username_len {
            self.log("auth_failure", Some(&user_log), "rejected: implausible username length");
            return (AuthOutcome::Failure { attempts_remaining: None }, None);
        }
        if req.secret.len() > self.policy.max_secret_len {
            self.log("auth_failure", Some(&user_log), "rejected: implausible secret length");
            return (AuthOutcome::Failure { attempts_remaining: None }, None);
        }
        match req.method {
            AuthMethod::Password if !self.policy.allow_password => {
                return (AuthOutcome::Error("password sign-in is disabled".into()), None)
            }
            AuthMethod::Pin if !self.policy.allow_pin => {
                return (AuthOutcome::Error("PIN sign-in is disabled".into()), None)
            }
            _ => {}
        }
        if req.user == "root" && self.policy.restrict_root_to_console && !is_console_tty(&req.tty) {
            self.log("auth_failure", Some("root"), "rejected: root login outside console seat");
            return (AuthOutcome::Failure { attempts_remaining: None }, None);
        }

        {
            let lockout = self.lockout.borrow();
            if let Err(remaining) = lockout.check(&req.user) {
                self.log("lockout", Some(&user_log), format!("blocked, {}s remaining", remaining.as_secs()));
                return (AuthOutcome::LockedOut { retry_after_secs: remaining.as_secs() }, None);
            }
        }

        self.log("auth_attempt", Some(&user_log), req.method.prompt_label());
        let ok = self.check_credential(&req);

        let mut lockout = self.lockout.borrow_mut();
        match ok {
            Ok(Some(session)) => {
                let _ = lockout.record_success(&req.user);
                drop(lockout);
                self.log("auth_success", Some(&user_log), req.method.prompt_label());
                (AuthOutcome::Success, session)
            }
            Ok(None) => {
                let _ = lockout.record_failure(&req.user);
                let attempts = lockout.check(&req.user);
                drop(lockout);
                self.log("auth_failure", Some(&user_log), "bad credential");
                match attempts {
                    Err(remaining) => (AuthOutcome::LockedOut { retry_after_secs: remaining.as_secs() }, None),
                    Ok(()) => (AuthOutcome::Failure { attempts_remaining: None }, None),
                }
            }
            Err(msg) => {
                drop(lockout);
                self.log("auth_failure", Some(&user_log), format!("backend error: {}", msg));
                (AuthOutcome::Error("authentication service unavailable".into()), None)
            }
        }
    }

    /// `Ok(Some(session))` on success with the PAM backend, `Ok(Some(_))`
    /// is never returned for the shadow backend (there is no PAM session to
    /// hand back), `Ok(None)` for a plain credential rejection, `Err` for a
    /// backend-level failure.
    fn check_credential(&self, req: &AuthRequest) -> Result<Option<Option<PamSession>>, String> {
        match &self.backend {
            Backend::Pam(api) => self.check_via_pam(api, req),
            Backend::ShadowOnly(crypt) => self.check_via_shadow(crypt, req),
            Backend::Unavailable => Err("no authentication backend loaded".to_string()),
        }
    }

    fn check_via_pam(&self, api: &Arc<PamApi>, req: &AuthRequest) -> Result<Option<Option<PamSession>>, String> {
        // req.secret is moved into the conversation object and lives exactly
        // as long as the PAM transaction needs it.
        let secret = Secret::new(req.secret.expose().to_vec());
        let conv = Box::new(SingleShotConv { secret: Some(secret) });
        let mut session = PamSession::start(api, req.method.pam_service(), &req.user, conv)
            .map_err(|e| e.to_string())?;
        session.set_item_str(PAM_TTY, &req.tty);
        let rc = session.authenticate(PAM_SILENT | PAM_DISALLOW_NULL_AUTHTOK);
        if rc != PAM_SUCCESS {
            return Ok(None);
        }
        let rc = session.acct_mgmt(PAM_SILENT);
        if rc != PAM_SUCCESS {
            // Account-management failure (expired, locked, ...) is a
            // credential rejection, not a backend error.
            return Ok(None);
        }
        let _ = session.setcred(PAM_ESTABLISH_CRED);
        // The live handle is handed all the way out to the launcher; it is
        // NOT dropped here.
        Ok(Some(Some(session)))
    }

    fn check_via_shadow(&self, crypt: &Arc<Crypt>, req: &AuthRequest) -> Result<Option<Option<PamSession>>, String> {
        if req.method != AuthMethod::Password {
            return Err("the shadow fallback only supports password sign-in".to_string());
        }
        match shadow::verify(crypt, &req.user, req.secret.expose()).map_err(|e| e.to_string())? {
            ShadowResult::Ok => Ok(Some(None)),
            ShadowResult::WrongPassword | ShadowResult::NoSuchUser | ShadowResult::AccountLocked | ShadowResult::AccountExpired => {
                Ok(None)
            }
        }
    }
}

fn is_console_tty(tty: &str) -> bool {
    tty.starts_with("tty") && tty[3..].chars().all(|c| c.is_ascii_digit()) && tty.len() > 3
}

/// Picks the strongest backend actually available on this system: real PAM
/// first, `crypt(3)`-against-shadow if libpam cannot be loaded at all. Never
/// silently invents a third option.
pub fn detect_backend(explicit_pam_library: &str) -> Backend {
    match PamApi::load(explicit_pam_library) {
        Ok(api) => Backend::Pam(Arc::new(api)),
        Err(_) => match Crypt::open() {
            Ok(c) => Backend::ShadowOnly(Arc::new(c)),
            Err(_) => Backend::Unavailable,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::audit::AuditLog;
    use std::path::PathBuf;

    fn tmp_authenticator(backend: Backend) -> (Authenticator, PathBuf) {
        let dir = std::env::temp_dir().join(format!("mitos-login-authtest-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos()));
        std::fs::create_dir_all(&dir).unwrap();
        let lockout = Lockout::open(&dir, SecurityPolicy::default()).unwrap();
        let audit = AuditLog::open(&dir.join("audit.jsonl")).unwrap();
        (Authenticator::new(backend, SecurityPolicy::default(), lockout, audit), dir)
    }

    #[test]
    fn rejects_implausible_input_before_touching_any_backend() {
        let (auth, dir) = tmp_authenticator(Backend::Unavailable);
        let req = AuthRequest {
            user: "a".repeat(500),
            method: AuthMethod::Password,
            secret: Secret::from_str("x"),
            tty: "tty1".into(),
        };
        match auth.authenticate(req).0 {
            AuthOutcome::Failure { .. } => {}
            other => panic!("expected Failure, got {:?}", other),
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn root_is_rejected_off_console() {
        let (auth, dir) = tmp_authenticator(Backend::Unavailable);
        let req = AuthRequest {
            user: "root".into(),
            method: AuthMethod::Password,
            secret: Secret::from_str("x"),
            tty: "pts/0".into(),
        };
        match auth.authenticate(req).0 {
            AuthOutcome::Failure { .. } => {}
            other => panic!("expected Failure for root off console, got {:?}", other),
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn console_tty_matcher() {
        assert!(is_console_tty("tty1"));
        assert!(is_console_tty("tty12"));
        assert!(!is_console_tty("pts/0"));
        assert!(!is_console_tty("tty"));
        assert!(!is_console_tty("ttyUSB0"));
    }

    #[test]
    fn unavailable_backend_is_a_hard_error_not_a_silent_pass() {
        let (auth, dir) = tmp_authenticator(Backend::Unavailable);
        let req = AuthRequest {
            user: "someone".into(),
            method: AuthMethod::Password,
            secret: Secret::from_str("x"),
            tty: "tty1".into(),
        };
        match auth.authenticate(req).0 {
            AuthOutcome::Error(_) => {}
            other => panic!("expected Error, got {:?}", other),
        }
        std::fs::remove_dir_all(&dir).ok();
    }
}
