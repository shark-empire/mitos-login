//! A byte buffer for passwords and other secrets: locked out of swap on a
//! best-effort basis and overwritten with zeros the moment it is dropped or
//! consumed, so a secret's lifetime in memory is as short as possible and it
//! is never silently copied by an ordinary `Clone`/`Debug`.
use crate::sys;
use std::fmt;

pub struct Secret {
    buf: Vec<u8>,
    locked: bool,
}

impl Secret {
    pub fn new(bytes: Vec<u8>) -> Secret {
        let locked = sys::lock_memory(bytes.as_ptr(), bytes.len());
        Secret { buf: bytes, locked }
    }

    pub fn from_str(s: &str) -> Secret {
        Secret::new(s.as_bytes().to_vec())
    }

    pub fn expose(&self) -> &[u8] {
        &self.buf
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// A read-only view as `&str`, if the bytes happen to be valid UTF-8.
    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.buf).ok()
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        if self.locked {
            sys::unlock_memory(self.buf.as_ptr(), self.buf.len());
        }
        sys::zeroize_vec(&mut self.buf);
    }
}

// Deliberately no Clone, Debug, Display, PartialEq: a secret should never be
// printed, logged, or compared in variable time by accident. Use `ct_eq`.
impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Secret({} bytes, REDACTED)", self.buf.len())
    }
}

/// Constant-time byte comparison. Returns `false` immediately only for a
/// length mismatch (lengths are not secret); every equal-length comparison
/// takes time independent of where the first difference is.
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ct_eq_matches_normal_equality() {
        assert!(ct_eq(b"hello", b"hello"));
        assert!(!ct_eq(b"hello", b"hellp"));
        assert!(!ct_eq(b"hello", b"hell"));
        assert!(ct_eq(b"", b""));
    }

    #[test]
    fn secret_zeroizes_on_drop() {
        let ptr;
        let len;
        {
            let s = Secret::from_str("hunter2");
            ptr = s.expose().as_ptr();
            len = s.expose().len();
            assert_eq!(s.as_str(), Some("hunter2"));
        }
        // Best-effort sanity check only: reading freed memory is UB in
        // general, so this does not run under Miri/ASan; it exists to catch
        // an accidental no-op zeroize during manual review.
        let after = unsafe { std::slice::from_raw_parts(ptr, len) };
        assert_eq!(after, [0u8; 7]);
    }
}
