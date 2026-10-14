//! The `Daemon` struct: owns every long-lived resource and runs the single
//! poll loop that drives the whole login flow. See docs/architecture.md.
use super::event_loop::PollSet;
use super::lifecycle::{self, GreeterHandle};
use crate::authentication::{AuthMethod, AuthOutcome, AuthRequest, Authenticator};
use crate::config::Config;
use crate::errors::{Error, Result};
use crate::ipc::messages::{AuthOutcomeWire, ControlRequest, ControlResponse, GreeterEvent, GreeterRequest};
use crate::ipc::protocol::{decode_greeter_request, encode_greeter_event, read_frame, write_frame};
use crate::ipc::server::ControlServer;
use crate::security::audit::AuditLog;
use crate::security::lockout::Lockout;
use crate::security::secret::Secret;
use crate::seats::{devices, SeatManager};
use crate::sessions::selection::{self, SessionType};
use crate::sessions::SessionManager;
use crate::sys::{self, ExitStatus};
use crate::users::UserManager;
use std::collections::BTreeMap;
use std::os::fd::AsRawFd;
use std::path::PathBuf;

pub struct Daemon {
    config: Config,
    config_dir: PathBuf,
    authenticator: Authenticator,
    seat_manager: SeatManager,
    display_fd: std::os::fd::RawFd,
    _display_file: std::fs::File,
    input_devices: Vec<devices::InputDevice>,
    session_manager: SessionManager,
    control_server: ControlServer,
    user_manager: UserManager,
    sessions: BTreeMap<String, SessionType>,
    power_caps: crate::power::Capabilities,
    signals: sys::SignalSource,
    greeter: Option<GreeterHandle>,
    shutting_down: bool,
}

const SHUTDOWN_SIGNALS: &[i32] = &[sys::SIGTERM, sys::SIGINT, sys::SIGHUP, sys::SIGCHLD, sys::SIGUSR1, sys::SIGUSR2];

impl Daemon {
    pub fn start(config_dir: &std::path::Path) -> Result<Daemon> {
        // Block/capture signals before anything else stands up, so nothing
        // can arrive and be silently lost (the default disposition) before
        // the signalfd exists to catch it.
        let signals = sys::SignalSource::new(SHUTDOWN_SIGNALS)?;
        sys::harden_process()?;

        let config = Config::load(config_dir)?;
        std::fs::create_dir_all(&config.state_dir)?;

        let mut audit = AuditLog::open(&config.state_dir.join("audit.jsonl"))?;
        audit.log("daemon_start", None, format!("mitos-login {}", crate::VERSION));

        let mut lockout = Lockout::open(&config.state_dir, config.security.clone())?;
        if lockout.tamper_detected {
            audit.log("integrity_violation", None, "lockout state failed signature check on startup; resetting");
            lockout.reset_after_tamper()?;
        }

        let backend = crate::authentication::client::detect_backend(&config.pam_library);
        let backend_name = match &backend {
            crate::authentication::client::Backend::Pam(_) => "pam",
            crate::authentication::client::Backend::ShadowOnly(_) => "shadow",
            crate::authentication::client::Backend::Unavailable => "unavailable",
        };
        audit.log("daemon_start", None, format!("authentication backend: {}", backend_name));
        let authenticator = Authenticator::new(backend, config.security.clone(), lockout, audit);

        let user_manager = UserManager::new(PathBuf::from("/etc/passwd"), config.user_list.clone());
        let sessions = selection::discover(&config.sessions_dir);
        let power_caps = crate::power::detect_capabilities();

        let control_server = ControlServer::bind(&config.control_socket_path)?;

        let mut seat_manager = SeatManager::new();
        seat_manager.acquire(config.greeter_vt)?;

        let display_file = std::fs::OpenOptions::new().read(true).write(true).open(&config.framebuffer_device)?;
        let display_fd = display_file.as_raw_fd();
        let _ = sys::fb_info(display_fd)?; // fail fast if this isn't really a framebuffer

        let input_devices = devices::open_all(&config.input_device_dir)?;

        let mut daemon = Daemon {
            config,
            config_dir: config_dir.to_path_buf(),
            authenticator,
            seat_manager,
            display_fd,
            _display_file: display_file,
            input_devices,
            session_manager: SessionManager::new(),
            control_server,
            user_manager,
            sessions,
            power_caps,
            signals,
            greeter: None,
            shutting_down: false,
        };

        if let Some(user) = daemon.config.autologin_user.clone() {
            if daemon.try_autologin(&user).is_ok() {
                return Ok(daemon);
            }
            // Falls through to a normal greeter on any autologin failure —
            // never leaves the machine with no way to sign in at all.
        }
        daemon.spawn_fresh_greeter()?;
        Ok(daemon)
    }

    fn input_fds(&self) -> Vec<std::os::fd::RawFd> {
        self.input_devices.iter().map(|d| d.file.as_raw_fd()).collect()
    }

    fn spawn_fresh_greeter(&mut self) -> Result<()> {
        let fds = self.input_fds();
        let handle = lifecycle::spawn_greeter(&self.config, &self.seat_manager, self.display_fd, &fds)?;
        self.greeter = Some(handle);
        Ok(())
    }

    fn try_autologin(&mut self, user: &str) -> Result<()> {
        let user = self
            .user_manager
            .resolve_for_login(user)?
            .ok_or_else(|| Error::config("autologin user does not exist"))?;
        let session_type = self.pick_session_for(&user)?;
        std::thread::sleep(self.config.autologin_delay);
        // Autologin skips password entry entirely but still goes through
        // PAM's account/session machinery (no PAM_AUTHTOK check requested)
        // so nss/pam_limits/pam_env etc. still apply normally.
        let outcome_session = match crate::authentication::client::detect_backend(&self.config.pam_library) {
            crate::authentication::client::Backend::Pam(api) => {
                let mut session = crate::sys::pam::PamSession::start(&api, "mitos-login-autologin", &user.name, Box::new(NullConv))?;
                let rc = session.acct_mgmt(crate::sys::pam::PAM_SILENT);
                if rc != crate::sys::pam::PAM_SUCCESS {
                    // A locked or expired account does not get a free pass
                    // just because autologin skips the password itself.
                    return Err(Error::auth(format!("autologin account rejected by PAM ({})", session.strerror(rc))));
                }
                let _ = session.setcred(crate::sys::pam::PAM_ESTABLISH_CRED);
                Some(session)
            }
            _ => None,
        };
        self.launch_session(&user, &session_type, outcome_session, "seat0", "autologin")
    }

    fn pick_session_for(&self, user: &crate::users::User) -> Result<SessionType> {
        if user.uid == 0 {
            if let Some(root_session) = &self.config.root_session {
                return Ok(SessionType { id: "root-shell".into(), name: "Administrator Shell".into(), exec: root_session.clone() });
            }
            return Ok(SessionType { id: "root-shell".into(), name: "Administrator Shell".into(), exec: user.shell.clone() });
        }
        self.sessions
            .values()
            .next()
            .cloned()
            .ok_or_else(|| Error::config("no *.session files found under sessions_dir"))
    }

    fn launch_session(
        &mut self,
        user: &crate::users::User,
        session_type: &SessionType,
        pam_session: Option<crate::sys::pam::PamSession>,
        seat: &str,
        tty: &str,
    ) -> Result<()> {
        let id = self.session_manager.launch(user, session_type, pam_session, &self.config.runtime_dir_base, seat, tty, self.config.language.code())?;
        self.authenticator.log_event("session_start", Some(&user.name), format!("session {} on {}", id, seat));
        self.seat_manager.release()?;
        if let Some(mut handle) = self.greeter.take() {
            let _ = write_frame(&mut handle.link, &encode_greeter_event(&GreeterEvent::Farewell));
            let _ = sys::kill(handle.child.id() as i32, sys::SIGTERM);
        }
        Ok(())
    }

    pub fn run_forever(&mut self) -> i32 {
        loop {
            if self.shutting_down {
                return 0;
            }
            let mut polls = PollSet::new();
            polls.add("signals", self.signals.fd());
            polls.add("control", self.control_server.fd());
            if let Some(g) = &self.greeter {
                polls.add("greeter", g.link.as_raw_fd());
            }
            if let Err(e) = polls.wait(1000) {
                eprintln!("mitos-login: poll error: {}", e);
                continue;
            }
            if polls.is_ready("signals") {
                self.handle_signals();
            }
            if self.shutting_down {
                continue;
            }
            for (stream, req) in self.control_server.accept_requests() {
                let response = self.handle_control(req);
                self.control_server.respond(stream, &response);
            }
            if polls.is_ready("greeter") {
                self.handle_greeter_readable();
            }
            if self.greeter.is_none() && !self.shutting_down && !self.session_manager.has_active_session_on("seat0") {
                // The greeter exited (crashed or was torn down) and no
                // session took over the seat -- get a fresh one up rather
                // than leaving the machine with no way to sign in.
                if self.seat_manager.seat().is_none() {
                    let _ = self.seat_manager.acquire(self.config.greeter_vt);
                }
                if let Err(e) = self.spawn_fresh_greeter() {
                    eprintln!("mitos-login: failed to respawn greeter: {}", e);
                }
            }
        }
    }

    fn handle_signals(&mut self) {
        for sig in self.signals.drain() {
            match sig {
                s if s == sys::SIGTERM || s == sys::SIGINT => {
                    self.begin_shutdown();
                }
                s if s == sys::SIGCHLD => self.reap_children(),
                s if s == sys::SIGHUP => self.reload_config(),
                _ => {}
            }
        }
    }

    fn reap_children(&mut self) {
        while let Ok(Some((pid, status))) = sys::wait_any_nonblocking() {
            if let Some(g) = &self.greeter {
                if g.child.id() as i32 == pid {
                    let outcome: String = match status {
                        ExitStatus::Exited(0) => "exited cleanly".to_string(),
                        ExitStatus::Exited(c) => format!("exited with code {}", c),
                        ExitStatus::Signaled(s) => format!("terminated by signal {}", s),
                    };
                    self.authenticator.log_event("session_failed", None, format!("greeter {}", outcome));
                    self.greeter = None;
                    continue;
                }
            }
            if let Some(session) = self.session_manager.reap(pid) {
                self.authenticator.log_event("session_end", Some(&session.user), format!("session {} ended", session.id));
                if self.seat_manager.seat().is_none() {
                    let _ = self.seat_manager.acquire(self.config.greeter_vt);
                }
            }
        }
    }

    fn begin_shutdown(&mut self) {
        self.shutting_down = true;
        if let Some(handle) = &self.greeter {
            let _ = sys::kill(handle.child.id() as i32, sys::SIGTERM);
        }
        let _ = self.seat_manager.release();
        self.authenticator.log_event("daemon_stop", None, "received shutdown signal");
    }

    fn reload_config(&mut self) {
        match Config::load(&self.config_dir) {
            Ok(new_cfg) => {
                self.user_manager = UserManager::new(PathBuf::from("/etc/passwd"), new_cfg.user_list.clone());
                self.sessions = selection::discover(&new_cfg.sessions_dir);
                self.authenticator.log_event("admin_command", None, "config reloaded");
                self.config = new_cfg;
            }
            Err(e) => {
                self.authenticator.log_event("admin_command", None, format!("config reload FAILED, keeping previous config: {}", e));
            }
        }
    }

    fn handle_control(&mut self, req: ControlRequest) -> ControlResponse {
        match req {
            ControlRequest::Status => ControlResponse::Status {
                version: crate::VERSION.to_string(),
                active_sessions: self.session_manager.active_sessions().len() as u32,
                backend: "see audit log at daemon_start".to_string(),
            },
            ControlRequest::ListSessions => ControlResponse::Sessions(
                self.session_manager.active_sessions().iter().map(|s| (s.id, s.user.clone(), s.seat.clone())).collect(),
            ),
            ControlRequest::Reload => {
                self.reload_config();
                ControlResponse::Ok
            }
            ControlRequest::VerifyAuditLog => match self.authenticator.verify_audit_log() {
                Ok(true) => ControlResponse::AuditOk,
                Ok(false) => ControlResponse::AuditTampered,
                Err(e) => ControlResponse::Error(e.to_string()),
            },
            ControlRequest::Terminate { session_id } => {
                let pid = self.session_manager.active_sessions().iter().find(|s| s.id == session_id).map(|s| s.pid);
                match pid {
                    // The actual bookkeeping cleanup (closing the PAM
                    // session, freeing the tracked slot) happens through the
                    // normal SIGCHLD -> reap_children path once this signal
                    // takes effect, exactly as if the session had exited on
                    // its own.
                    Some(pid) => match sys::kill(pid, sys::SIGTERM) {
                        Ok(()) => ControlResponse::Ok,
                        Err(e) => ControlResponse::Error(e.to_string()),
                    },
                    None => ControlResponse::Error("no such session".to_string()),
                }
            }
        }
    }

    fn handle_greeter_readable(&mut self) {
        let Some(handle) = self.greeter.as_mut() else { return };
        let frame = read_frame(&mut handle.link);
        let req = match frame.and_then(|b| decode_greeter_request(&b)) {
            Ok(r) => r,
            Err(_) => return, // malformed frame from our own greeter: ignore this message, keep the link
        };
        match req {
            GreeterRequest::Ready => self.send_show_event(),
            GreeterRequest::Authenticate { user, method, secret } => self.handle_authenticate(user, method, secret),
            GreeterRequest::SelectSession { .. } => {}
            GreeterRequest::PowerAction(action) => self.handle_power_action(action),
            GreeterRequest::Heartbeat => {}
        }
    }

    fn send_show_event(&mut self) {
        if self.greeter.as_ref().is_some_and(|g| g.sent_show) {
            return;
        }
        let (users, sessions) = match lifecycle::build_show_event(&self.user_manager, &self.sessions, &self.config_dir) {
            Ok(v) => v,
            Err(_) => (Vec::new(), Vec::new()),
        };
        if let Some(handle) = self.greeter.as_mut() {
            let _ = write_frame(&mut handle.link, &encode_greeter_event(&GreeterEvent::Show { users, sessions, last_user: None }));
            let _ = write_frame(
                &mut handle.link,
                &encode_greeter_event(&GreeterEvent::PowerCapabilities {
                    can_suspend: self.power_caps.can_suspend,
                    can_reboot: self.power_caps.can_reboot,
                    can_poweroff: self.power_caps.can_poweroff,
                }),
            );
            handle.sent_show = true;
        }
    }

    fn handle_authenticate(&mut self, user: String, method: AuthMethod, secret: Vec<u8>) {
        let tty = self.seat_manager.seat().map(|s| s.tty_name()).unwrap_or_else(|| "tty1".to_string());
        let req = AuthRequest { user: user.clone(), method, secret: Secret::new(secret), tty };
        let (outcome, pam_session) = self.authenticator.authenticate(req);
        let wire = AuthOutcomeWire::from(&outcome);
        if let Some(handle) = self.greeter.as_mut() {
            let _ = write_frame(&mut handle.link, &encode_greeter_event(&GreeterEvent::AuthFeedback { outcome: wire }));
        }
        if outcome == AuthOutcome::Success {
            if let Ok(Some(resolved)) = self.user_manager.resolve_for_login(&user) {
                let session_type = match self.pick_session_for(&resolved) {
                    Ok(s) => s,
                    Err(e) => {
                        self.authenticator.log_event("session_failed", Some(&user), format!("no session available: {}", e));
                        return;
                    }
                };
                let seat_name = self.seat_manager.seat().map(|s| s.name.clone()).unwrap_or_else(|| "seat0".to_string());
                let tty_name = self.seat_manager.seat().map(|s| s.tty_name()).unwrap_or_default();
                if let Err(e) = self.launch_session(&resolved, &session_type, pam_session, &seat_name, &tty_name) {
                    self.authenticator.log_event("session_failed", Some(&user), e.to_string());
                }
            }
        }
    }

    fn handle_power_action(&mut self, action: crate::ipc::messages::PowerAction) {
        if !lifecycle::power_action_allowed(action) {
            return;
        }
        self.authenticator.log_event("admin_command", None, format!("power action {:?} requested from greeter", action));
        if let Err(e) = crate::power::client::dispatch(action, &self.power_caps) {
            if let Some(handle) = self.greeter.as_mut() {
                let _ = write_frame(&mut handle.link, &encode_greeter_event(&GreeterEvent::PowerActionFailed { message: e.to_string() }));
            }
        }
        // A successful reboot/poweroff/suspend either never returns (the
        // machine is going down) or returns after a resume, in which case
        // the greeter is exactly where it should still be -- nothing further
        // to do here either way.
    }
}

/// PAM conversation used for autologin: PAM should not need to prompt for
/// anything since no credential check is requested, but the callback must
/// still exist and safely refuse if a module somehow does ask.
struct NullConv;
impl crate::sys::pam::PamConv for NullConv {
    fn prompt(&mut self, _echo: bool, _text: &str) -> Option<Secret> {
        None
    }
    fn info(&mut self, _text: &str) {}
    fn error(&mut self, _text: &str) {}
}
