//! Authorization for the control socket: `SO_PEERCRED` is a kernel-verified
//! fact about the connecting process, not something the peer can lie about
//! over the wire — this is the actual security boundary, not the socket's
//! file permissions (which are still set to `0600` as defense in depth).
use crate::sys::{self, PeerCred};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdminAuth {
    Allowed,
    Denied { uid: u32 },
}

pub fn check(stream: &UnixStream) -> crate::errors::Result<AdminAuth> {
    let PeerCred { uid, .. } = sys::peer_credentials(stream.as_raw_fd())?;
    Ok(if uid == 0 { AdminAuth::Allowed } else { AdminAuth::Denied { uid } })
}
