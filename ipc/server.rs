//! The daemon side of the control socket: bind, listen, and (once the
//! daemon's poll loop sees a readable listener) accept one connection,
//! authorize it, read one request, dispatch, reply, close. One request per
//! connection, matching mitos-services' own control-socket convention.
use super::permissions::{self, AdminAuth};
use super::protocol::{decode_control_request, encode_control_response, read_frame, write_frame};
use super::{ControlRequest, ControlResponse};
use crate::errors::Result;
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

pub struct ControlServer {
    listener: UnixListener,
}

impl ControlServer {
    /// Binds the socket at `path`, replacing a stale one left behind by a
    /// previous crashed instance (a live daemon is never running while this
    /// runs, so there is no other legitimate owner to race with).
    pub fn bind(path: &Path) -> Result<ControlServer> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let _ = std::fs::remove_file(path);
        let listener = UnixListener::bind(path)?;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        listener.set_nonblocking(true)?;
        Ok(ControlServer { listener })
    }

    pub fn fd(&self) -> RawFd {
        self.listener.as_raw_fd()
    }

    /// Accepts every currently-pending connection, authorizes each one
    /// (`SO_PEERCRED` -- see [`permissions`]) and reads its one request.
    /// Deliberately returns the still-open streams paired with their
    /// decoded request rather than computing a response itself: the caller
    /// (the daemon's main loop) may need `&mut self` to answer some
    /// requests (e.g. `Reload`), and holding that borrow here while this
    /// method also borrows `self.listener` would conflict. An unauthorized
    /// or malformed connection is answered immediately with an error and
    /// never appears in the returned list.
    pub fn accept_requests(&self) -> Vec<(UnixStream, ControlRequest)> {
        let mut out = Vec::new();
        loop {
            let (mut stream, _addr) = match self.listener.accept() {
                Ok(pair) => pair,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            };
            let auth = match permissions::check(&stream) {
                Ok(a) => a,
                Err(_) => continue,
            };
            if let AdminAuth::Denied { .. } = auth {
                let _ = stream.set_nonblocking(false);
                let _ = write_frame(&mut stream, &encode_control_response(&ControlResponse::Error("not permitted".to_string())));
                continue;
            }
            let _ = stream.set_nonblocking(false);
            match read_frame(&mut stream).and_then(|b| decode_control_request(&b)) {
                Ok(req) => out.push((stream, req)),
                Err(_) => {
                    let _ = write_frame(&mut stream, &encode_control_response(&ControlResponse::Error("malformed request".to_string())));
                }
            }
        }
        out
    }

    /// Sends `response` on a stream obtained from [`Self::accept_requests`]
    /// and closes it (one request per connection, by design).
    pub fn respond(&self, mut stream: UnixStream, response: &ControlResponse) {
        let _ = write_frame(&mut stream, &encode_control_response(response));
    }
}
