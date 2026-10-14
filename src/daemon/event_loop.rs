//! A minimal `poll(2)`-based multiplexer: register fds once by name, poll
//! once per iteration, then ask "was this one readable?" — small enough that
//! pulling in a real async runtime for a single-threaded daemon this size
//! would only add weight, not clarity.
use crate::sys::{self, PollFd, POLLIN};
use std::os::fd::RawFd;

pub struct PollSet {
    names: Vec<&'static str>,
    fds: Vec<PollFd>,
}

impl PollSet {
    pub fn new() -> PollSet {
        PollSet { names: Vec::new(), fds: Vec::new() }
    }

    pub fn add(&mut self, name: &'static str, fd: RawFd) {
        self.names.push(name);
        self.fds.push(PollFd { fd, events: POLLIN, revents: 0 });
    }

    /// Waits up to `timeout_ms` (`-1` to wait indefinitely) for any
    /// registered fd to become readable.
    pub fn wait(&mut self, timeout_ms: i32) -> crate::errors::Result<()> {
        for pfd in self.fds.iter_mut() {
            pfd.revents = 0;
        }
        sys::poll(&mut self.fds, timeout_ms)?;
        Ok(())
    }

    pub fn is_ready(&self, name: &str) -> bool {
        self.names
            .iter()
            .zip(self.fds.iter())
            .any(|(n, f)| *n == name && (f.revents & POLLIN) != 0)
    }
}

impl Default for PollSet {
    fn default() -> Self {
        Self::new()
    }
}
