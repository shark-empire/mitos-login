//! Raw libc declarations. Only x86_64/aarch64 Linux (glibc or musl) is supported;
//! every value below is checked against the real system headers by
//! `scripts/check-ffi.sh` (see docs/security.md).
use std::os::raw::{c_char, c_int, c_ulong, c_void};

pub const EPERM: c_int = 1;
pub const ENOENT: c_int = 2;
pub const ESRCH: c_int = 3;
pub const EINTR: c_int = 4;
pub const ECHILD: c_int = 10;
pub const EBADF: c_int = 9;
pub const EAGAIN: c_int = 11;
pub const ERANGE: c_int = 34;

pub const POLLIN: i16 = 0x001;
pub const POLLOUT: i16 = 0x004;
pub const POLLERR: i16 = 0x008;
pub const POLLHUP: i16 = 0x010;
pub const POLLNVAL: i16 = 0x020;

pub const SIG_BLOCK: c_int = 0;
pub const SIG_SETMASK: c_int = 2;
pub const SFD_NONBLOCK: c_int = 0o4000;
pub const SFD_CLOEXEC: c_int = 0o2000000;
pub const SIGHUP: c_int = 1;
pub const SIGINT: c_int = 2;
pub const SIGQUIT: c_int = 3;
pub const SIGKILL: c_int = 9;
pub const SIGUSR1: c_int = 10;
pub const SIGUSR2: c_int = 12;
pub const SIGTERM: c_int = 15;
pub const SIGCHLD: c_int = 17;
pub const WNOHANG: c_int = 1;

pub const SOL_SOCKET: c_int = 1;
pub const SO_PEERCRED: c_int = 17;

pub const PR_SET_PDEATHSIG: c_int = 1;
pub const PR_SET_DUMPABLE: c_int = 4;
pub const PR_SET_NO_NEW_PRIVS: c_int = 38;
pub const RLIMIT_CORE: c_int = 4;

pub const F_DUPFD_CLOEXEC: c_int = 1030;
pub const O_RDWR: c_int = 2;
pub const O_NOCTTY: c_int = 0o400;
pub const TIOCSCTTY: c_ulong = 0x540E;

pub const PROT_READ: c_int = 1;
pub const PROT_WRITE: c_int = 2;
pub const MAP_SHARED: c_int = 1;

pub const RTLD_NOW: c_int = 2;

pub const VT_OPENQRY: c_ulong = 0x5600;
pub const VT_GETSTATE: c_ulong = 0x5603;
pub const VT_ACTIVATE: c_ulong = 0x5606;
pub const VT_WAITACTIVE: c_ulong = 0x5607;
pub const VT_LOCKSWITCH: c_ulong = 0x560B;
pub const VT_UNLOCKSWITCH: c_ulong = 0x560C;
pub const KDSETMODE: c_ulong = 0x4B3A;
pub const KDSKBMODE: c_ulong = 0x4B45;
pub const KDGKBMODE: c_ulong = 0x4B44;
pub const FBIOGET_VSCREENINFO: c_ulong = 0x4600;
pub const FBIOGET_FSCREENINFO: c_ulong = 0x4602;
pub const EVIOCGRAB: c_ulong = 0x40044590;

/// Rebuilds the Linux `_IOC(dir, type, nr, size)` encoding used by the
/// variable-length evdev ioctls (`include/uapi/asm-generic/ioctl.h`).
/// Verified against the real macros for concrete sizes by `scripts/check-ffi.sh`.
pub const fn ioc_read(ty: u8, nr: u8, size: usize) -> c_ulong {
    const IOC_READ: c_ulong = 2;
    ((IOC_READ << 30) | ((ty as c_ulong) << 8) | (nr as c_ulong) | ((size as c_ulong) << 16)) as c_ulong
}

/// `EVIOCGNAME(len)`.
pub const fn eviocgname(len: usize) -> c_ulong {
    ioc_read(b'E', 0x06, len)
}

/// `EVIOCGBIT(ev, len)`.
pub const fn eviocgbit(ev: u16, len: usize) -> c_ulong {
    ioc_read(b'E', 0x20 + (ev as u8), len)
}


pub const ECHO: u32 = 0o10;
pub const TCSANOW: c_int = 0;
pub const LFLAG_INDEX: usize = 3;

pub const RB_AUTOBOOT: c_int = 0x01234567;
pub const RB_POWER_OFF: c_int = 0x4321fedc;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VtStat {
    pub v_active: u16,
    pub v_signal: u16,
    pub v_state: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct InputEvent {
    pub tv_sec: i64,
    pub tv_usec: i64,
    pub type_: u16,
    pub code: u16,
    pub value: i32,
}

pub const EV_SYN: u16 = 0x00;
pub const EV_KEY: u16 = 0x01;
pub const EV_REL: u16 = 0x02;
pub const REL_X: u16 = 0x00;
pub const REL_Y: u16 = 0x01;
pub const REL_WHEEL: u16 = 0x08;
pub const BTN_LEFT: u16 = 0x110;
pub const BTN_RIGHT: u16 = 0x111;
pub const BTN_MIDDLE: u16 = 0x112;

pub const KEY_ESC: u16 = 1;
pub const KEY_1: u16 = 2;
pub const KEY_0: u16 = 11;
pub const KEY_MINUS: u16 = 12;
pub const KEY_EQUAL: u16 = 13;
pub const KEY_BACKSPACE: u16 = 14;
pub const KEY_TAB: u16 = 15;
pub const KEY_Q: u16 = 16;
pub const KEY_P: u16 = 25;
pub const KEY_LEFTBRACE: u16 = 26;
pub const KEY_RIGHTBRACE: u16 = 27;
pub const KEY_ENTER: u16 = 28;
pub const KEY_LEFTCTRL: u16 = 29;
pub const KEY_A: u16 = 30;
pub const KEY_L: u16 = 38;
pub const KEY_SEMICOLON: u16 = 39;
pub const KEY_APOSTROPHE: u16 = 40;
pub const KEY_GRAVE: u16 = 41;
pub const KEY_LEFTSHIFT: u16 = 42;
pub const KEY_BACKSLASH: u16 = 43;
pub const KEY_Z: u16 = 44;
pub const KEY_M: u16 = 50;
pub const KEY_COMMA: u16 = 51;
pub const KEY_DOT: u16 = 52;
pub const KEY_SLASH: u16 = 53;
pub const KEY_RIGHTSHIFT: u16 = 54;
pub const KEY_KPASTERISK: u16 = 55;
pub const KEY_LEFTALT: u16 = 56;
pub const KEY_SPACE: u16 = 57;
pub const KEY_CAPSLOCK: u16 = 58;
pub const KEY_F1: u16 = 59;
pub const KEY_F10: u16 = 68;
pub const KEY_KP7: u16 = 71;
pub const KEY_KP8: u16 = 72;
pub const KEY_KP9: u16 = 73;
pub const KEY_KPMINUS: u16 = 74;
pub const KEY_KP4: u16 = 75;
pub const KEY_KP5: u16 = 76;
pub const KEY_KP6: u16 = 77;
pub const KEY_KPPLUS: u16 = 78;
pub const KEY_KP1: u16 = 79;
pub const KEY_KP2: u16 = 80;
pub const KEY_KP3: u16 = 81;
pub const KEY_KP0: u16 = 82;
pub const KEY_KPDOT: u16 = 83;
pub const KEY_102ND: u16 = 86;
pub const KEY_F11: u16 = 87;
pub const KEY_F12: u16 = 88;
pub const KEY_KPENTER: u16 = 96;
pub const KEY_RIGHTCTRL: u16 = 97;
pub const KEY_KPSLASH: u16 = 98;
pub const KEY_RIGHTALT: u16 = 100;
pub const KEY_HOME: u16 = 102;
pub const KEY_UP: u16 = 103;
pub const KEY_PAGEUP: u16 = 104;
pub const KEY_LEFT: u16 = 105;
pub const KEY_RIGHT: u16 = 106;
pub const KEY_END: u16 = 107;
pub const KEY_DOWN: u16 = 108;
pub const KEY_PAGEDOWN: u16 = 109;
pub const KEY_INSERT: u16 = 110;
pub const KEY_DELETE: u16 = 111;
pub const KEY_LEFTMETA: u16 = 125;
pub const KEY_RIGHTMETA: u16 = 126;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PollFd {
    pub fd: c_int,
    pub events: i16,
    pub revents: i16,
}

/// Opaque `sigset_t` (128 bytes on glibc and musl).
#[repr(C)]
pub struct SigSet {
    pub bits: [u64; 16],
}

#[repr(C)]
pub struct RLimit {
    pub cur: u64,
    pub max: u64,
}

#[repr(C)]
pub struct UCred {
    pub pid: i32,
    pub uid: u32,
    pub gid: u32,
}

#[repr(C)]
pub struct Passwd {
    pub pw_name: *mut c_char,
    pub pw_passwd: *mut c_char,
    pub pw_uid: u32,
    pub pw_gid: u32,
    pub pw_gecos: *mut c_char,
    pub pw_dir: *mut c_char,
    pub pw_shell: *mut c_char,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FbBitfield {
    pub offset: u32,
    pub length: u32,
    pub msb_right: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FbVarScreeninfo {
    pub xres: u32,
    pub yres: u32,
    pub xres_virtual: u32,
    pub yres_virtual: u32,
    pub xoffset: u32,
    pub yoffset: u32,
    pub bits_per_pixel: u32,
    pub grayscale: u32,
    pub red: FbBitfield,
    pub green: FbBitfield,
    pub blue: FbBitfield,
    pub transp: FbBitfield,
    pub nonstd: u32,
    pub activate: u32,
    pub height: u32,
    pub width: u32,
    pub accel_flags: u32,
    pub pixclock: u32,
    pub left_margin: u32,
    pub right_margin: u32,
    pub upper_margin: u32,
    pub lower_margin: u32,
    pub hsync_len: u32,
    pub vsync_len: u32,
    pub sync: u32,
    pub vmode: u32,
    pub rotate: u32,
    pub colorspace: u32,
    pub reserved: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FbFixScreeninfo {
    pub id: [u8; 16],
    pub smem_start: c_ulong,
    pub smem_len: u32,
    pub type_: u32,
    pub type_aux: u32,
    pub visual: u32,
    pub xpanstep: u16,
    pub ypanstep: u16,
    pub ywrapstep: u16,
    pub line_length: u32,
    pub mmio_start: c_ulong,
    pub mmio_len: u32,
    pub accel: u32,
    pub capabilities: u16,
    pub reserved: [u16; 2],
}

extern "C" {
    pub fn getuid() -> u32;
    pub fn geteuid() -> u32;
    pub fn getpid() -> i32;
    pub fn getppid() -> i32;
    pub fn umask(mask: u32) -> u32;
    pub fn prctl(option: c_int, ...) -> c_int;
    pub fn setrlimit(resource: c_int, rlim: *const RLimit) -> c_int;
    pub fn mlock(addr: *const c_void, len: usize) -> c_int;
    pub fn munlock(addr: *const c_void, len: usize) -> c_int;
    pub fn poll(fds: *mut PollFd, nfds: c_ulong, timeout: c_int) -> c_int;
    pub fn sigemptyset(set: *mut SigSet) -> c_int;
    pub fn sigaddset(set: *mut SigSet, signum: c_int) -> c_int;
    pub fn pthread_sigmask(how: c_int, set: *const SigSet, old: *mut SigSet) -> c_int;
    pub fn signalfd(fd: c_int, mask: *const SigSet, flags: c_int) -> c_int;
    pub fn kill(pid: i32, sig: c_int) -> c_int;
    pub fn waitpid(pid: i32, status: *mut c_int, options: c_int) -> i32;
    pub fn getsockopt(fd: c_int, level: c_int, name: c_int, val: *mut c_void, len: *mut u32) -> c_int;
    pub fn getpwnam_r(
        name: *const c_char,
        pwd: *mut Passwd,
        buf: *mut c_char,
        buflen: usize,
        result: *mut *mut Passwd,
    ) -> c_int;
    pub fn getpwuid_r(
        uid: u32,
        pwd: *mut Passwd,
        buf: *mut c_char,
        buflen: usize,
        result: *mut *mut Passwd,
    ) -> c_int;
    pub fn getgrouplist(user: *const c_char, group: u32, groups: *mut u32, ngroups: *mut c_int) -> c_int;
    pub fn setgroups(size: usize, list: *const u32) -> c_int;
    pub fn setgid(gid: u32) -> c_int;
    pub fn setuid(uid: u32) -> c_int;
    pub fn setsid() -> i32;
    pub fn dup2(old: c_int, new: c_int) -> c_int;
    pub fn fcntl(fd: c_int, cmd: c_int, ...) -> c_int;
    pub fn chdir(path: *const c_char) -> c_int;
    pub fn chown(path: *const c_char, owner: u32, group: u32) -> c_int;
    pub fn fchown(fd: c_int, owner: u32, group: u32) -> c_int;
    pub fn fchmod(fd: c_int, mode: u32) -> c_int;
    pub fn open(path: *const c_char, flags: c_int, ...) -> c_int;
    pub fn close(fd: c_int) -> c_int;
    pub fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
    pub fn mmap(addr: *mut c_void, len: usize, prot: c_int, flags: c_int, fd: c_int, off: i64) -> *mut c_void;
    pub fn munmap(addr: *mut c_void, len: usize) -> c_int;
    pub fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    pub fn dlclose(handle: *mut c_void) -> c_int;
    pub fn dlerror() -> *mut c_char;
    pub fn malloc(size: usize) -> *mut c_void;
    pub fn calloc(n: usize, size: usize) -> *mut c_void;
    pub fn free(p: *mut c_void);
    pub fn tcgetattr(fd: c_int, t: *mut c_void) -> c_int;
    pub fn tcsetattr(fd: c_int, action: c_int, t: *const c_void) -> c_int;
    pub fn reboot(cmd: c_int) -> c_int;
    pub fn sync();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn struct_sizes_match_the_kernel_abi() {
        assert_eq!(size_of::<PollFd>(), 8);
        assert_eq!(size_of::<SigSet>(), 128);
        assert_eq!(size_of::<RLimit>(), 16);
        assert_eq!(size_of::<UCred>(), 12);
        assert_eq!(size_of::<Passwd>(), 48);
        assert_eq!(size_of::<FbVarScreeninfo>(), 160);
        assert_eq!(size_of::<FbFixScreeninfo>(), 80);
    }
}
