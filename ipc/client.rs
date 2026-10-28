//! The `mitos-loginctl` side of the control socket: one request, one
//! response, then the connection closes.
use super::protocol::{decode_control_response, encode_control_request, read_frame, write_frame};
use super::{ControlRequest, ControlResponse};
use crate::errors::Result;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

pub fn call(socket_path: &Path, req: &ControlRequest) -> Result<ControlResponse> {
    let mut stream = UnixStream::connect(socket_path)?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
    write_frame(&mut stream, &encode_control_request(req))?;
    let bytes = read_frame(&mut stream)?;
    decode_control_response(&bytes)
}
