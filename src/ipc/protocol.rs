//! Length-prefixed framing plus a small hand-rolled binary encoding for
//! [`super::messages`]. Every read is bounds-checked and returns
//! [`Error::protocol`] rather than panicking: the peer on the other end of
//! either link must never be able to crash this process, however malformed
//! its bytes are.
use super::messages::*;
use crate::authentication::AuthMethod;
use crate::errors::{Error, Result};
use std::io::{Read, Write};

/// A single frame's payload may not exceed this — generous for anything this
/// protocol actually sends, and small enough that a misbehaving peer cannot
/// make the reader allocate an unreasonable amount of memory from one
/// length-prefix value alone.
pub const MAX_FRAME_LEN: u32 = 1 << 20;

pub struct Writer(Vec<u8>);

impl Writer {
    pub fn new() -> Writer {
        Writer(Vec::new())
    }
    pub fn u8(&mut self, v: u8) -> &mut Self {
        self.0.push(v);
        self
    }
    pub fn u32(&mut self, v: u32) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    pub fn u64(&mut self, v: u64) -> &mut Self {
        self.0.extend_from_slice(&v.to_le_bytes());
        self
    }
    pub fn bool(&mut self, v: bool) -> &mut Self {
        self.u8(v as u8)
    }
    pub fn bytes(&mut self, v: &[u8]) -> &mut Self {
        self.u32(v.len() as u32);
        self.0.extend_from_slice(v);
        self
    }
    pub fn str(&mut self, v: &str) -> &mut Self {
        self.bytes(v.as_bytes())
    }
    pub fn opt_str(&mut self, v: &Option<String>) -> &mut Self {
        match v {
            Some(s) => {
                self.bool(true);
                self.str(s);
            }
            None => {
                self.bool(false);
            }
        }
        self
    }
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl Default for Writer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Reader<'a> {
        Reader { buf, pos: 0 }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.pos.checked_add(n).ok_or_else(|| Error::protocol("length overflow"))?;
        let slice = self.buf.get(self.pos..end).ok_or_else(|| Error::protocol("frame ended early"))?;
        self.pos = end;
        Ok(slice)
    }

    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    pub fn u32(&mut self) -> Result<u32> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    pub fn u64(&mut self) -> Result<u64> {
        let b = self.take(8)?;
        Ok(u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]))
    }
    pub fn bool(&mut self) -> Result<bool> {
        Ok(self.u8()? != 0)
    }
    pub fn bytes(&mut self) -> Result<Vec<u8>> {
        let len = self.u32()?;
        if len > MAX_FRAME_LEN {
            return Err(Error::protocol("field length too large"));
        }
        Ok(self.take(len as usize)?.to_vec())
    }
    pub fn str(&mut self) -> Result<String> {
        let bytes = self.bytes()?;
        String::from_utf8(bytes).map_err(|_| Error::protocol("field is not valid UTF-8"))
    }
    pub fn opt_str(&mut self) -> Result<Option<String>> {
        if self.bool()? {
            Ok(Some(self.str()?))
        } else {
            Ok(None)
        }
    }
    pub fn finish(&self) -> Result<()> {
        if self.pos == self.buf.len() {
            Ok(())
        } else {
            Err(Error::protocol("trailing bytes after message"))
        }
    }
}

fn auth_method_tag(m: AuthMethod) -> u8 {
    match m {
        AuthMethod::Password => 0,
        AuthMethod::Pin => 1,
    }
}
fn auth_method_from_tag(t: u8) -> Result<AuthMethod> {
    match t {
        0 => Ok(AuthMethod::Password),
        1 => Ok(AuthMethod::Pin),
        _ => Err(Error::protocol("unknown auth method tag")),
    }
}

fn outcome_tag(o: &AuthOutcomeWire) -> u8 {
    match o {
        AuthOutcomeWire::Success => 0,
        AuthOutcomeWire::Failure => 1,
        AuthOutcomeWire::LockedOut { .. } => 2,
        AuthOutcomeWire::Error => 3,
    }
}

fn power_action_tag(a: PowerAction) -> u8 {
    match a {
        PowerAction::Suspend => 0,
        PowerAction::Reboot => 1,
        PowerAction::PowerOff => 2,
    }
}
fn power_action_from_tag(t: u8) -> Result<PowerAction> {
    match t {
        0 => Ok(PowerAction::Suspend),
        1 => Ok(PowerAction::Reboot),
        2 => Ok(PowerAction::PowerOff),
        _ => Err(Error::protocol("unknown power action tag")),
    }
}

// ---------------------------------------------------------------- GreeterEvent

pub fn encode_greeter_event(ev: &GreeterEvent) -> Vec<u8> {
    let mut w = Writer::new();
    match ev {
        GreeterEvent::Show { users, sessions, last_user } => {
            w.u8(0);
            w.u32(users.len() as u32);
            for u in users {
                w.str(&u.name).str(&u.display_name).u32(u.uid).bool(u.has_avatar).bool(u.locked);
            }
            w.u32(sessions.len() as u32);
            for s in sessions {
                w.str(&s.id).str(&s.name);
            }
            w.opt_str(last_user);
        }
        GreeterEvent::AuthFeedback { outcome } => {
            w.u8(1).u8(outcome_tag(outcome));
            if let AuthOutcomeWire::LockedOut { retry_after_secs } = outcome {
                w.u64(*retry_after_secs);
            }
        }
        GreeterEvent::Shake => {
            w.u8(2);
        }
        GreeterEvent::PowerCapabilities { can_suspend, can_reboot, can_poweroff } => {
            w.u8(3).bool(*can_suspend).bool(*can_reboot).bool(*can_poweroff);
        }
        GreeterEvent::PowerActionFailed { message } => {
            w.u8(4).str(message);
        }
        GreeterEvent::Farewell => {
            w.u8(5);
        }
    }
    w.into_bytes()
}

pub fn decode_greeter_event(buf: &[u8]) -> Result<GreeterEvent> {
    let mut r = Reader::new(buf);
    let ev = match r.u8()? {
        0 => {
            let n = r.u32()? as usize;
            if n > 100_000 {
                return Err(Error::protocol("implausible user count"));
            }
            let mut users = Vec::with_capacity(n.min(1024));
            for _ in 0..n {
                let name = r.str()?;
                let display_name = r.str()?;
                let uid = r.u32()?;
                let has_avatar = r.bool()?;
                let locked = r.bool()?;
                users.push(UserSummary { name, display_name, uid, has_avatar, locked });
            }
            let sn = r.u32()? as usize;
            if sn > 100_000 {
                return Err(Error::protocol("implausible session count"));
            }
            let mut sessions = Vec::with_capacity(sn.min(1024));
            for _ in 0..sn {
                let id = r.str()?;
                let name = r.str()?;
                sessions.push(SessionSummary { id, name });
            }
            let last_user = r.opt_str()?;
            GreeterEvent::Show { users, sessions, last_user }
        }
        1 => {
            let tag = r.u8()?;
            let outcome = match tag {
                0 => AuthOutcomeWire::Success,
                1 => AuthOutcomeWire::Failure,
                2 => AuthOutcomeWire::LockedOut { retry_after_secs: r.u64()? },
                3 => AuthOutcomeWire::Error,
                _ => return Err(Error::protocol("unknown outcome tag")),
            };
            GreeterEvent::AuthFeedback { outcome }
        }
        2 => GreeterEvent::Shake,
        3 => GreeterEvent::PowerCapabilities {
            can_suspend: r.bool()?,
            can_reboot: r.bool()?,
            can_poweroff: r.bool()?,
        },
        4 => GreeterEvent::PowerActionFailed { message: r.str()? },
        5 => GreeterEvent::Farewell,
        _ => return Err(Error::protocol("unknown GreeterEvent tag")),
    };
    r.finish()?;
    Ok(ev)
}

// ---------------------------------------------------------------- GreeterRequest

pub fn encode_greeter_request(req: &GreeterRequest) -> Vec<u8> {
    let mut w = Writer::new();
    match req {
        GreeterRequest::Ready => {
            w.u8(0);
        }
        GreeterRequest::Authenticate { user, method, secret } => {
            w.u8(1).str(user).u8(auth_method_tag(*method)).bytes(secret);
        }
        GreeterRequest::SelectSession { id } => {
            w.u8(2).str(id);
        }
        GreeterRequest::PowerAction(a) => {
            w.u8(3).u8(power_action_tag(*a));
        }
        GreeterRequest::Heartbeat => {
            w.u8(4);
        }
    }
    w.into_bytes()
}

pub fn decode_greeter_request(buf: &[u8]) -> Result<GreeterRequest> {
    let mut r = Reader::new(buf);
    let req = match r.u8()? {
        0 => GreeterRequest::Ready,
        1 => {
            let user = r.str()?;
            let method = auth_method_from_tag(r.u8()?)?;
            let secret = r.bytes()?;
            GreeterRequest::Authenticate { user, method, secret }
        }
        2 => GreeterRequest::SelectSession { id: r.str()? },
        3 => GreeterRequest::PowerAction(power_action_from_tag(r.u8()?)?),
        4 => GreeterRequest::Heartbeat,
        _ => return Err(Error::protocol("unknown GreeterRequest tag")),
    };
    r.finish()?;
    Ok(req)
}

// ---------------------------------------------------------------- Control protocol

pub fn encode_control_request(req: &ControlRequest) -> Vec<u8> {
    let mut w = Writer::new();
    match req {
        ControlRequest::Status => {
            w.u8(0);
        }
        ControlRequest::ListSessions => {
            w.u8(1);
        }
        ControlRequest::Reload => {
            w.u8(2);
        }
        ControlRequest::VerifyAuditLog => {
            w.u8(3);
        }
        ControlRequest::Terminate { session_id } => {
            w.u8(4).u32(*session_id);
        }
    }
    w.into_bytes()
}

pub fn decode_control_request(buf: &[u8]) -> Result<ControlRequest> {
    let mut r = Reader::new(buf);
    let req = match r.u8()? {
        0 => ControlRequest::Status,
        1 => ControlRequest::ListSessions,
        2 => ControlRequest::Reload,
        3 => ControlRequest::VerifyAuditLog,
        4 => ControlRequest::Terminate { session_id: r.u32()? },
        _ => return Err(Error::protocol("unknown ControlRequest tag")),
    };
    r.finish()?;
    Ok(req)
}

pub fn encode_control_response(resp: &ControlResponse) -> Vec<u8> {
    let mut w = Writer::new();
    match resp {
        ControlResponse::Status { version, active_sessions, backend } => {
            w.u8(0).str(version).u32(*active_sessions).str(backend);
        }
        ControlResponse::Sessions(list) => {
            w.u8(1).u32(list.len() as u32);
            for (id, user, seat) in list {
                w.u32(*id).str(user).str(seat);
            }
        }
        ControlResponse::Ok => {
            w.u8(2);
        }
        ControlResponse::AuditOk => {
            w.u8(3);
        }
        ControlResponse::AuditTampered => {
            w.u8(4);
        }
        ControlResponse::Error(msg) => {
            w.u8(5).str(msg);
        }
    }
    w.into_bytes()
}

pub fn decode_control_response(buf: &[u8]) -> Result<ControlResponse> {
    let mut r = Reader::new(buf);
    let resp = match r.u8()? {
        0 => ControlResponse::Status { version: r.str()?, active_sessions: r.u32()?, backend: r.str()? },
        1 => {
            let n = r.u32()? as usize;
            if n > 100_000 {
                return Err(Error::protocol("implausible session count"));
            }
            let mut list = Vec::with_capacity(n.min(1024));
            for _ in 0..n {
                list.push((r.u32()?, r.str()?, r.str()?));
            }
            ControlResponse::Sessions(list)
        }
        2 => ControlResponse::Ok,
        3 => ControlResponse::AuditOk,
        4 => ControlResponse::AuditTampered,
        5 => ControlResponse::Error(r.str()?),
        _ => return Err(Error::protocol("unknown ControlResponse tag")),
    };
    r.finish()?;
    Ok(resp)
}

// ---------------------------------------------------------------- framing I/O

/// Writes one length-prefixed frame. `payload.len()` must already be within
/// [`MAX_FRAME_LEN`] — both encoders above only ever produce messages far
/// smaller than that limit.
pub fn write_frame<W: Write>(w: &mut W, payload: &[u8]) -> Result<()> {
    if payload.len() as u64 > MAX_FRAME_LEN as u64 {
        return Err(Error::protocol("outgoing frame too large"));
    }
    // One buffer, one `write_all`: a reader polling the other end can never
    // observe the length prefix without (the start of) its payload.
    let mut buf = Vec::with_capacity(4 + payload.len());
    buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    buf.extend_from_slice(payload);
    let result = w.write_all(&buf);
    // A frame can carry a password: scrub the copy before it is freed.
    crate::sys::zeroize_vec(&mut buf);
    result?;
    Ok(())
}

/// Reads exactly one length-prefixed frame, rejecting an oversized
/// length-prefix before allocating a buffer for it.
pub fn read_frame<R: Read>(r: &mut R) -> Result<Vec<u8>> {
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf);
    if len > MAX_FRAME_LEN {
        return Err(Error::protocol("incoming frame too large"));
    }
    let mut buf = vec![0u8; len as usize];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

/// Incremental frame reader for a **non-blocking** stream. `read_exact` on a
/// non-blocking socket can consume part of a frame and then fail with
/// `WouldBlock`, silently losing those bytes and desynchronizing the link
/// for good; this keeps whatever has arrived buffered until a whole frame is
/// there. Used on both ends of the greeter link.
#[derive(Default)]
pub struct FrameReader {
    buf: Vec<u8>,
}

impl FrameReader {
    pub fn new() -> FrameReader {
        FrameReader { buf: Vec::new() }
    }

    /// Reads whatever is currently available (without blocking) and returns
    /// every complete frame now buffered, oldest first. `Err` means the link
    /// is unusable: the peer closed it, or sent an oversized length prefix.
    pub fn read_available<R: Read>(&mut self, r: &mut R) -> Result<Vec<Vec<u8>>> {
        let mut tmp = [0u8; 4096];
        let filled = self.fill(r, &mut tmp);
        // The scratch buffer saw every byte that arrived, passwords included.
        crate::sys::zeroize(&mut tmp);
        let eof = filled?;
        let mut frames = Vec::new();
        loop {
            if self.buf.len() < 4 {
                break;
            }
            let len = u32::from_le_bytes([self.buf[0], self.buf[1], self.buf[2], self.buf[3]]);
            if len > MAX_FRAME_LEN {
                return Err(Error::protocol("incoming frame too large"));
            }
            let end = 4 + len as usize;
            if self.buf.len() < end {
                break;
            }
            frames.push(self.buf[4..end].to_vec());
            crate::sys::zeroize(&mut self.buf[..end]);
            self.buf.drain(..end);
        }
        if eof && frames.is_empty() {
            return Err(Error::protocol("peer closed the link"));
        }
        Ok(frames)
    }

    /// Pulls bytes into `self.buf` until the stream would block, ends, or a
    /// fairness cap is hit. `Ok(true)` means end-of-stream.
    fn fill<R: Read>(&mut self, r: &mut R, tmp: &mut [u8; 4096]) -> Result<bool> {
        let mut total = 0usize;
        loop {
            match r.read(&mut tmp[..]) {
                Ok(0) => return Ok(true),
                Ok(n) => {
                    self.buf.extend_from_slice(&tmp[..n]);
                    total += n;
                    if total >= 64 * 1024 {
                        return Ok(false); // yield; the caller polls again
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(false),
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reader_rejects_short_input_without_panicking() {
        let mut r = Reader::new(&[1, 2]);
        assert!(r.u32().is_err());
    }

    #[test]
    fn reader_rejects_huge_length_prefix_before_allocating() {
        let mut buf = Writer::new();
        buf.u32(0xffff_ffff);
        let mut r = Reader::new(&buf.into_bytes());
        assert!(r.bytes().is_err());
    }

    #[test]
    fn round_trips_show_event() {
        let ev = GreeterEvent::Show {
            users: vec![UserSummary { name: "alice".into(), display_name: "Alice".into(), uid: 1000, has_avatar: true, locked: false }],
            sessions: vec![SessionSummary { id: "mitos".into(), name: "MITOS Desktop".into() }],
            last_user: Some("alice".into()),
        };
        let bytes = encode_greeter_event(&ev);
        assert_eq!(decode_greeter_event(&bytes).unwrap(), ev);
    }

    #[test]
    fn round_trips_authenticate_request_with_secret_bytes() {
        let req = GreeterRequest::Authenticate { user: "bob".into(), method: AuthMethod::Pin, secret: vec![1, 2, 3, 0, 255] };
        let bytes = encode_greeter_request(&req);
        assert_eq!(decode_greeter_request(&bytes).unwrap(), req);
    }

    #[test]
    fn trailing_bytes_are_rejected() {
        let mut bytes = encode_greeter_request(&GreeterRequest::Ready);
        bytes.push(0xff);
        assert!(decode_greeter_request(&bytes).is_err());
    }

    #[test]
    fn frame_round_trip_over_a_socketpair() {
        use std::os::unix::net::UnixStream;
        let (mut a, mut b) = UnixStream::pair().unwrap();
        let payload = encode_control_request(&ControlRequest::Terminate { session_id: 42 });
        write_frame(&mut a, &payload).unwrap();
        let got = read_frame(&mut b).unwrap();
        assert_eq!(decode_control_request(&got).unwrap(), ControlRequest::Terminate { session_id: 42 });
    }

    #[test]
    fn frame_reader_reassembles_a_frame_split_across_reads() {
        // A reader that hands back one byte per call, then WouldBlock.
        struct Drip { data: Vec<u8>, pos: usize }
        impl Read for Drip {
            fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
                if self.pos >= self.data.len() {
                    return Err(std::io::ErrorKind::WouldBlock.into());
                }
                out[0] = self.data[self.pos];
                self.pos += 1;
                Ok(1)
            }
        }
        let payload = encode_greeter_request(&GreeterRequest::Heartbeat);
        let mut wire = Vec::new();
        write_frame(&mut wire, &payload).unwrap();
        let mut reader = FrameReader::new();
        // First half only: nothing complete yet, nothing lost.
        let half = wire.len() / 2;
        let mut first = Drip { data: wire[..half].to_vec(), pos: 0 };
        assert!(reader.read_available(&mut first).unwrap().is_empty());
        // The rest arrives later: the whole frame comes out intact.
        let mut second = Drip { data: wire[half..].to_vec(), pos: 0 };
        let frames = reader.read_available(&mut second).unwrap();
        assert_eq!(frames, vec![payload]);
    }

    #[test]
    fn frame_reader_returns_several_queued_frames_in_order() {
        let mut wire = Vec::new();
        write_frame(&mut wire, &[1]).unwrap();
        write_frame(&mut wire, &[2, 2]).unwrap();
        let mut cursor = std::io::Cursor::new(wire);
        let mut reader = FrameReader::new();
        let frames = reader.read_available(&mut cursor).unwrap();
        assert_eq!(frames, vec![vec![1], vec![2, 2]]);
        // Cursor is now at EOF: the next call reports the closed link.
        assert!(reader.read_available(&mut cursor).is_err());
    }

    #[test]
    fn frame_reader_rejects_an_oversized_length_prefix() {
        let mut wire = Vec::new();
        wire.extend_from_slice(&(MAX_FRAME_LEN + 1).to_le_bytes());
        let mut cursor = std::io::Cursor::new(wire);
        assert!(FrameReader::new().read_available(&mut cursor).is_err());
    }
}
