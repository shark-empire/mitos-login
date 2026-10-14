//! Acquiring and releasing a virtual terminal for the greeter: switch to it,
//! stop the kernel's own text console from drawing or echoing keys over the
//! framebuffer, and lock switching so nothing (a stray Ctrl+Alt+F2, a
//! misbehaving client) can escape to a text VT mid-login.
use super::seat::Seat;
use crate::errors::Result;
use crate::sys;
use std::fs::OpenOptions;
use std::os::fd::AsRawFd;

pub struct SeatManager {
    seat: Option<Seat>,
    previous_kb_mode: i32,
}

const K_UNICODE: i32 = 3;
const K_OFF: i32 = 4;

impl SeatManager {
    pub fn new() -> SeatManager {
        SeatManager { seat: None, previous_kb_mode: K_UNICODE }
    }

    /// Finds a free VT, switches to it, and takes exclusive graphics-mode
    /// ownership. Idempotent: calling it again after [`Self::release`] is
    /// the normal way to reacquire the seat for a new greeter instance.
    pub fn acquire(&mut self, requested_vt: Option<u32>) -> Result<()> {
        let vt = match requested_vt {
            Some(v) => v,
            None => sys::vt_open_query()?,
        };
        let tty = OpenOptions::new().read(true).write(true).open(format!("/dev/tty{}", vt))?;
        let fd = tty.as_raw_fd();
        sys::vt_activate(fd, vt)?;
        sys::vt_wait_active(fd, vt)?;
        self.previous_kb_mode = sys::kb_get_mode(fd).unwrap_or(K_UNICODE);
        sys::kb_set_mode(fd, K_OFF)?;
        sys::kd_set_graphics(fd, true)?;
        sys::vt_lock_switch(fd, true)?;
        // The logical seat is always "seat0" (the same name logind uses), no
        // matter which VT it currently sits on: everything else (XDG_SEAT,
        // the daemon's "is a session running on this seat" check) compares
        // against that one stable name.
        self.seat = Some(Seat { vt_number: vt, tty, name: "seat0".to_string() });
        Ok(())
    }

    pub fn seat(&self) -> Option<&Seat> {
        self.seat.as_ref()
    }

    /// Restores the console to a normal text VT. Called before launching the
    /// user's session (which may want the VT for its own compositor) and on
    /// daemon shutdown.
    pub fn release(&mut self) -> Result<()> {
        if let Some(seat) = self.seat.take() {
            let fd = seat.tty.as_raw_fd();
            sys::vt_lock_switch(fd, false)?;
            sys::kd_set_graphics(fd, false)?;
            sys::kb_set_mode(fd, self.previous_kb_mode)?;
        }
        Ok(())
    }
}

impl Default for SeatManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SeatManager {
    fn drop(&mut self) {
        let _ = self.release();
    }
}
