//! The only module in the crate that may contain `unsafe`.
//!
//! Every wrapper here is small, documents its invariants, and exposes a safe
//! API. Constants and struct layouts in [`ffi`] are verified against the system
//! headers by `scripts/check-ffi.sh`.
#[cfg(not(all(target_os = "linux", any(target_arch = "x86_64", target_arch = "aarch64"))))]
compile_error!("mitos-login supports Linux on x86_64 and aarch64 only");

pub mod ffi;
pub mod pam;

use crate::errors::{Error, Result};
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::{self, Read};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::raw::{c_char, c_int, c_ulong, c_void};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};

pub use ffi::{PollFd, POLLERR, POLLHUP, POLLIN, POLLNVAL, POLLOUT};

pub const SIGHUP: i32 = ffi::SIGHUP;
pub const SIGINT: i32 = ffi::SIGINT;
pub const SIGQUIT: i32 = ffi::SIGQUIT;
pub const SIGKILL: i32 = ffi::SIGKILL;
pub const SIGUSR1: i32 = ffi::SIGUSR1;
pub const SIGUSR2: i32 = ffi::SIGUSR2;
pub const SIGTERM: i32 = ffi::SIGTERM;
pub const SIGCHLD: i32 = ffi::SIGCHLD;

// ---------------------------------------------------------------- basics

pub fn errno() -> i32 {
    io::Error::last_os_error().raw_os_error().unwrap_or(0)
}

pub fn getuid() -> u32 {
    unsafe { ffi::getuid() }
}

pub fn geteuid() -> u32 {
    unsafe { ffi::geteuid() }
}

pub fn getpid() -> i32 {
    unsafe { ffi::getpid() }
}

pub fn umask(mask: u32) -> u32 {
    unsafe { ffi::umask(mask & 0o777) }
}

/// Process-wide hardening for the daemon: no core dumps, not ptrace-able by
/// unprivileged users, restrictive umask.
pub fn harden_process() -> Result<()> {
    unsafe {
        if ffi::prctl(ffi::PR_SET_DUMPABLE, 0 as c_ulong, 0 as c_ulong, 0 as c_ulong, 0 as c_ulong) != 0 {
            return Err(Error::sys("prctl(PR_SET_DUMPABLE)"));
        }
        let lim = ffi::RLimit { cur: 0, max: 0 };
        if ffi::setrlimit(ffi::RLIMIT_CORE, &lim) != 0 {
            return Err(Error::sys("setrlimit(RLIMIT_CORE)"));
        }
    }
    umask(0o077);
    Ok(())
}

/// Overwrites `buf` with zeros in a way the optimizer may not remove.
pub fn zeroize(buf: &mut [u8]) {
    for b in buf.iter_mut() {
        unsafe { std::ptr::write_volatile(b as *mut u8, 0) };
    }
    std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
}

/// Zeroes the whole allocation of `v` (including spare capacity) and clears it.
pub fn zeroize_vec(v: &mut Vec<u8>) {
    let cap = v.capacity();
    let p = v.as_mut_ptr();
    for i in 0..cap {
        unsafe { std::ptr::write_volatile(p.add(i), 0) };
    }
    std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
    v.clear();
}

/// Best-effort `mlock`, used to keep secrets out of swap.
pub fn lock_memory(ptr: *const u8, len: usize) -> bool {
    if len == 0 {
        return true;
    }
    unsafe { ffi::mlock(ptr as *const c_void, len) == 0 }
}

pub fn unlock_memory(ptr: *const u8, len: usize) {
    if len > 0 {
        unsafe {
            ffi::munlock(ptr as *const c_void, len);
        }
    }
}

// ---------------------------------------------------------------- poll & signals

/// `poll(2)`. `EINTR` is reported as "nothing ready" so callers just loop.
pub fn poll(fds: &mut [PollFd], timeout_ms: i32) -> Result<usize> {
    let rc = unsafe { ffi::poll(fds.as_mut_ptr(), fds.len() as c_ulong, timeout_ms) };
    if rc < 0 {
        let e = errno();
        if e == ffi::EINTR {
            return Ok(0);
        }
        return Err(Error::Sys { call: "poll", errno: e });
    }
    Ok(rc as usize)
}

fn new_sigset(sigs: &[i32]) -> Result<ffi::SigSet> {
    let mut set = ffi::SigSet { bits: [0; 16] };
    unsafe {
        if ffi::sigemptyset(&mut set) != 0 {
            return Err(Error::sys("sigemptyset"));
        }
        for &s in sigs {
            if ffi::sigaddset(&mut set, s) != 0 {
                return Err(Error::sys("sigaddset"));
            }
        }
    }
    Ok(set)
}

/// Blocks `sigs` in the calling thread (and every thread it spawns later).
pub fn block_signals(sigs: &[i32]) -> Result<()> {
    let set = new_sigset(sigs)?;
    let rc = unsafe { ffi::pthread_sigmask(ffi::SIG_BLOCK, &set, std::ptr::null_mut()) };
    if rc != 0 {
        return Err(Error::Sys { call: "pthread_sigmask", errno: rc });
    }
    Ok(())
}

/// A `signalfd(2)` that turns asynchronous signals into pollable events.
pub struct SignalSource {
    file: File,
}

impl SignalSource {
    /// Blocks `sigs` and returns a source that yields them. Call this before
    /// spawning any thread so every thread inherits the blocked mask.
    pub fn new(sigs: &[i32]) -> Result<SignalSource> {
        block_signals(sigs)?;
        let set = new_sigset(sigs)?;
        let fd = unsafe { ffi::signalfd(-1, &set, ffi::SFD_CLOEXEC | ffi::SFD_NONBLOCK) };
        if fd < 0 {
            return Err(Error::sys("signalfd"));
        }
        let file = unsafe { File::from_raw_fd(fd) };
        Ok(SignalSource { file })
    }

    pub fn fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }

    /// Returns every signal number currently pending.
    pub fn drain(&self) -> Vec<i32> {
        let mut out = Vec::new();
        let mut buf = [0u8; 128];
        let mut reader = &self.file;
        loop {
            match reader.read(&mut buf) {
                Ok(128) => out.push(u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]) as i32),
                _ => break,
            }
            if out.len() > 256 {
                break;
            }
        }
        out
    }
}

pub fn kill(pid: i32, sig: i32) -> Result<()> {
    if unsafe { ffi::kill(pid, sig) } != 0 {
        return Err(Error::sys("kill"));
    }
    Ok(())
}

pub const WNOHANG: i32 = ffi::WNOHANG;

/// The outcome of a reaped child, decoded from the raw `wait(2)` status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitStatus {
    Exited(i32),
    Signaled(i32),
}

fn decode_status(status: i32) -> ExitStatus {
    let sig = status & 0x7f;
    if sig == 0 {
        ExitStatus::Exited((status >> 8) & 0xff)
    } else {
        ExitStatus::Signaled(sig)
    }
}

/// One non-blocking `waitpid(-1, ..., WNOHANG)` call: reaps at most one
/// already-exited child of this process, of any pid. `Ok(None)` means
/// nothing was ready; call this in a loop from a `SIGCHLD`-driven poll until
/// it returns `Ok(None)` to drain every child that exited since the signal.
pub fn wait_any_nonblocking() -> Result<Option<(i32, ExitStatus)>> {
    let mut status: c_int = 0;
    let pid = unsafe { ffi::waitpid(-1, &mut status, ffi::WNOHANG) };
    if pid > 0 {
        Ok(Some((pid, decode_status(status))))
    } else if pid == 0 {
        Ok(None)
    } else {
        let e = errno();
        if e == ffi::ECHILD {
            // No children left at all — not an error for a poll loop.
            Ok(None)
        } else {
            Err(Error::Sys { call: "waitpid", errno: e })
        }
    }
}

// ---------------------------------------------------------------- peer credentials, users, groups

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PeerCred {
    pub pid: i32,
    pub uid: u32,
    pub gid: u32,
}

/// Kernel-verified credentials of the process that connected a Unix socket.
pub fn peer_credentials(fd: RawFd) -> Result<PeerCred> {
    let mut c = ffi::UCred { pid: 0, uid: 0, gid: 0 };
    let mut len = std::mem::size_of::<ffi::UCred>() as u32;
    let rc = unsafe {
        ffi::getsockopt(
            fd,
            ffi::SOL_SOCKET,
            ffi::SO_PEERCRED,
            &mut c as *mut ffi::UCred as *mut c_void,
            &mut len,
        )
    };
    if rc != 0 || len as usize != std::mem::size_of::<ffi::UCred>() {
        return Err(Error::sys("getsockopt(SO_PEERCRED)"));
    }
    Ok(PeerCred { pid: c.pid, uid: c.uid, gid: c.gid })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasswdEntry {
    pub name: String,
    pub uid: u32,
    pub gid: u32,
    pub gecos: String,
    pub dir: String,
    pub shell: String,
}

fn cstr_field(p: *const c_char) -> String {
    if p.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
    }
}

fn pw_lookup<F>(f: F) -> Result<Option<PasswdEntry>>
where
    F: Fn(*mut ffi::Passwd, *mut c_char, usize, *mut *mut ffi::Passwd) -> c_int,
{
    let mut size = 1024usize;
    loop {
        let mut buf: Vec<u8> = vec![0; size];
        let mut pwd: ffi::Passwd = unsafe { std::mem::zeroed() };
        let mut result: *mut ffi::Passwd = std::ptr::null_mut();
        let rc = f(&mut pwd, buf.as_mut_ptr() as *mut c_char, size, &mut result);
        if rc == ffi::ERANGE && size < (1 << 20) {
            size *= 2;
            continue;
        }
        if rc != 0 {
            if rc == ffi::ENOENT || rc == ffi::ESRCH || rc == ffi::EBADF || rc == ffi::EPERM {
                return Ok(None);
            }
            return Err(Error::Sys { call: "getpwnam_r", errno: rc });
        }
        if result.is_null() {
            return Ok(None);
        }
        return Ok(Some(PasswdEntry {
            name: cstr_field(pwd.pw_name),
            uid: pwd.pw_uid,
            gid: pwd.pw_gid,
            gecos: cstr_field(pwd.pw_gecos),
            dir: cstr_field(pwd.pw_dir),
            shell: cstr_field(pwd.pw_shell),
        }));
    }
}

/// Looks a user up through NSS (files, LDAP, ...).
pub fn getpwnam(name: &str) -> Result<Option<PasswdEntry>> {
    let c = CString::new(name).map_err(|_| Error::invalid("NUL byte in user name"))?;
    pw_lookup(|pwd, buf, len, res| unsafe { ffi::getpwnam_r(c.as_ptr(), pwd, buf, len, res) })
}

pub fn getpwuid(uid: u32) -> Result<Option<PasswdEntry>> {
    pw_lookup(|pwd, buf, len, res| unsafe { ffi::getpwuid_r(uid, pwd, buf, len, res) })
}

/// Supplementary group list of `name` (including `gid`), as `getgrouplist(3)`.
pub fn group_list(name: &str, gid: u32) -> Result<Vec<u32>> {
    let c = CString::new(name).map_err(|_| Error::invalid("NUL byte in user name"))?;
    let mut n: c_int = 32;
    for _ in 0..5 {
        if n <= 0 || n > 65536 {
            break;
        }
        let mut groups: Vec<u32> = vec![0; n as usize];
        let mut count: c_int = n;
        let rc = unsafe { ffi::getgrouplist(c.as_ptr(), gid, groups.as_mut_ptr(), &mut count) };
        if rc >= 0 {
            let keep = if count < 0 { 0 } else { count as usize };
            groups.truncate(keep);
            return Ok(groups);
        }
        n = if count > n { count } else { n.saturating_mul(2) };
    }
    Err(Error::session("getgrouplist failed"))
}

// ---------------------------------------------------------------- dynamic libraries

/// A `dlopen`ed shared library.
pub struct Library {
    handle: *mut c_void,
}

// A dlopen handle is process-global and may be used from any thread.
unsafe impl Send for Library {}
unsafe impl Sync for Library {}

impl Library {
    pub fn open(name: &str) -> Result<Library> {
        let c = CString::new(name).map_err(|_| Error::invalid("NUL byte in library name"))?;
        let handle = unsafe { ffi::dlopen(c.as_ptr(), ffi::RTLD_NOW) };
        if handle.is_null() {
            let msg = unsafe {
                let e = ffi::dlerror();
                if e.is_null() {
                    String::from("unknown error")
                } else {
                    CStr::from_ptr(e).to_string_lossy().into_owned()
                }
            };
            return Err(Error::Load(format!("{}: {}", name, msg)));
        }
        Ok(Library { handle })
    }

    pub fn open_any(names: &[&str]) -> Result<Library> {
        let mut last: Option<Error> = None;
        for n in names {
            match Library::open(n) {
                Ok(l) => return Ok(l),
                Err(e) => last = Some(e),
            }
        }
        Err(last.unwrap_or_else(|| Error::Load("no library candidates".to_string())))
    }

    pub fn symbol(&self, name: &str) -> Result<*mut c_void> {
        let c = CString::new(name).map_err(|_| Error::invalid("NUL byte in symbol name"))?;
        let p = unsafe { ffi::dlsym(self.handle, c.as_ptr()) };
        if p.is_null() {
            return Err(Error::Load(format!("missing symbol {}", name)));
        }
        Ok(p)
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe {
            ffi::dlclose(self.handle);
        }
    }
}

fn is_trusted_file(path: &str) -> bool {
    use std::os::unix::fs::MetadataExt;
    match std::fs::metadata(path) {
        Ok(m) => m.is_file() && m.uid() == 0 && (m.mode() & 0o022) == 0,
        Err(_) => false,
    }
}

/// Candidate paths for a system library: root-owned, non-writable files in the
/// standard directories first, then the bare soname as a last resort.
pub fn system_library_candidates(soname: &str) -> Vec<String> {
    let dirs = [
        "/lib/x86_64-linux-gnu",
        "/usr/lib/x86_64-linux-gnu",
        "/lib/aarch64-linux-gnu",
        "/usr/lib/aarch64-linux-gnu",
        "/usr/lib64",
        "/lib64",
        "/usr/lib",
        "/lib",
    ];
    let mut out = Vec::new();
    for d in dirs.iter() {
        let p = format!("{}/{}", d, soname);
        if is_trusted_file(&p) {
            out.push(p);
        }
    }
    out.push(soname.to_string());
    out
}

/// `crypt(3)` loaded lazily from libcrypt. Not thread-safe: keep it inside the
/// single authentication worker thread.
type CryptFn = unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_char;

pub struct Crypt {
    _lib: Library,
    f: CryptFn,
}

unsafe impl Send for Crypt {}

impl Crypt {
    pub fn open() -> Result<Crypt> {
        let mut names: Vec<String> = system_library_candidates("libcrypt.so.1");
        names.extend(system_library_candidates("libcrypt.so.2"));
        let refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        let lib = Library::open_any(&refs)?;
        let p = lib.symbol("crypt")?;
        let f: CryptFn = unsafe { std::mem::transmute::<*mut c_void, CryptFn>(p) };
        Ok(Crypt { _lib: lib, f })
    }

    /// Constant-time check of `password` against a crypt(3) `stored` hash.
    pub fn verify(&self, password: &[u8], stored: &str) -> bool {
        if password.contains(&0) || stored.as_bytes().contains(&0) || stored.len() < 4 {
            return false;
        }
        let pw = match CString::new(password) {
            Ok(c) => c,
            Err(_) => return false,
        };
        let st = match CString::new(stored) {
            Ok(c) => c,
            Err(_) => return false,
        };
        let out = unsafe { (self.f)(pw.as_ptr(), st.as_ptr()) };
        let mut pw_bytes = pw.into_bytes();
        zeroize(&mut pw_bytes);
        if out.is_null() {
            return false;
        }
        let got = unsafe { CStr::from_ptr(out) }.to_bytes();
        if got.first() == Some(&b'*') {
            return false;
        }
        crate::security::secret::ct_eq(got, stored.as_bytes())
    }
}

// ---------------------------------------------------------------- spawning

#[derive(Clone, Debug)]
pub struct Creds {
    pub uid: u32,
    pub gid: u32,
    pub groups: Vec<u32>,
}

/// Everything needed to start a child with a controlled environment.
#[derive(Clone, Debug)]
pub struct SpawnSpec {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub cwd: String,
    /// `None` keeps the current credentials.
    pub creds: Option<Creds>,
    pub new_session: bool,
    pub umask: u32,
    pub no_new_privs: bool,
    pub die_with_parent: bool,
    /// A terminal device (e.g. `/dev/tty3`) to make the child's controlling
    /// terminal and its stdin/stdout/stderr -- what a console login shell
    /// needs. Opened as root *before* the credential drop, then handed to
    /// `creds` (mode 0600). Requires `new_session`.
    pub tty: Option<String>,
    /// `(source fd in parent, destination fd in child)`; everything else is closed on exec.
    pub fds: Vec<(RawFd, RawFd)>,
}

impl SpawnSpec {
    pub fn new(program: &str) -> SpawnSpec {
        SpawnSpec {
            program: program.to_string(),
            args: Vec::new(),
            env: Vec::new(),
            cwd: "/".to_string(),
            creds: None,
            new_session: false,
            umask: 0o077,
            no_new_privs: false,
            die_with_parent: false,
            tty: None,
            fds: Vec::new(),
        }
    }
}

/// Runs in the forked child before `exec`. Only async-signal-safe calls, no
/// allocation. The credential drop order is groups, gid, uid, then a check that
/// root cannot be regained.
unsafe fn child_setup(
    new_session: bool,
    tty: &Option<CString>,
    map: &[(RawFd, RawFd)],
    creds: &Option<Creds>,
    cwd: &CStr,
    umask_v: u32,
    no_new_privs: bool,
    die_with_parent: bool,
    parent: i32,
) -> io::Result<()> {
    // The daemon blocks several signals (SIGTERM/SIGCHLD/...) to read them
    // through a signalfd, and a blocked-signal mask survives `exec`. Start
    // every child with an empty one, or a greeter/session could never be
    // terminated or notice its own children exiting. (std's `Command`
    // already does this before `pre_exec` closures run on Linux; doing it
    // again here keeps that guarantee from depending on a std internal.)
    let empty = ffi::SigSet { bits: [0; 16] };
    if ffi::pthread_sigmask(ffi::SIG_SETMASK, &empty, std::ptr::null_mut()) != 0 {
        return Err(io::Error::last_os_error());
    }
    if new_session && ffi::setsid() < 0 {
        return Err(io::Error::last_os_error());
    }
    if let Some(path) = tty {
        // A console login: the terminal becomes this session's controlling
        // terminal and its stdio. Done here, while still root -- a tty node
        // belongs to root until it is handed over just below.
        let fd = ffi::open(path.as_ptr(), ffi::O_RDWR | ffi::O_NOCTTY);
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        if ffi::ioctl(fd, ffi::TIOCSCTTY, 0 as c_ulong) < 0 {
            return Err(io::Error::last_os_error());
        }
        if let Some(c) = creds {
            if ffi::fchown(fd, c.uid, c.gid) != 0 {
                return Err(io::Error::last_os_error());
            }
        }
        if ffi::fchmod(fd, 0o600) != 0 {
            return Err(io::Error::last_os_error());
        }
        for target in 0..3 {
            if ffi::dup2(fd, target) < 0 {
                return Err(io::Error::last_os_error());
            }
        }
        if fd > 2 {
            ffi::close(fd);
        }
    }
    for &(src, dst) in map.iter() {
        if ffi::dup2(src, dst) < 0 {
            return Err(io::Error::last_os_error());
        }
    }
    if let Some(c) = creds {
        if ffi::setgroups(c.groups.len(), c.groups.as_ptr()) != 0 {
            return Err(io::Error::last_os_error());
        }
        if ffi::setgid(c.gid) != 0 {
            return Err(io::Error::last_os_error());
        }
        if ffi::setuid(c.uid) != 0 {
            return Err(io::Error::last_os_error());
        }
        if c.uid != 0 && (ffi::setuid(0) == 0 || ffi::setgid(0) == 0) {
            return Err(io::Error::from_raw_os_error(ffi::EPERM));
        }
    }
    if ffi::chdir(cwd.as_ptr()) != 0 {
        let _ = ffi::chdir(b"/\0".as_ptr() as *const c_char);
    }
    ffi::umask(umask_v);
    if no_new_privs
        && ffi::prctl(ffi::PR_SET_NO_NEW_PRIVS, 1 as c_ulong, 0 as c_ulong, 0 as c_ulong, 0 as c_ulong) != 0
    {
        return Err(io::Error::last_os_error());
    }
    if die_with_parent {
        if ffi::prctl(ffi::PR_SET_PDEATHSIG, ffi::SIGKILL as c_ulong, 0 as c_ulong, 0 as c_ulong, 0 as c_ulong) != 0 {
            return Err(io::Error::last_os_error());
        }
        if ffi::getppid() != parent {
            return Err(io::Error::from_raw_os_error(ffi::ESRCH));
        }
    }
    Ok(())
}

/// Spawns `spec` with a cleared environment, null stdio, and (optionally) a
/// full privilege drop performed in the child before `exec`.
pub fn spawn(spec: &SpawnSpec) -> Result<Child> {
    let cwd_c = CString::new(spec.cwd.as_str()).map_err(|_| Error::invalid("NUL byte in working directory"))?;
    // Move every source descriptor to >= 10 first. dup2 destinations are
    // assigned sequentially (pair k -> fd k+3), so a destination is always
    // numerically below the source of any *later* pair (sources are >= 10+k),
    // which means no dup2 can overwrite a source that is still waiting.
    let mut keep_alive: Vec<OwnedFd> = Vec::new();
    let mut map: Vec<(RawFd, RawFd)> = Vec::new();
    for &(src, dst) in spec.fds.iter() {
        let hi = unsafe { ffi::fcntl(src, ffi::F_DUPFD_CLOEXEC, 10 as c_int) };
        if hi < 0 {
            return Err(Error::sys("fcntl(F_DUPFD_CLOEXEC)"));
        }
        keep_alive.push(unsafe { OwnedFd::from_raw_fd(hi) });
        map.push((hi, dst));
    }
    let mut cmd = Command::new(&spec.program);
    cmd.args(&spec.args).env_clear();
    for (k, v) in spec.env.iter() {
        cmd.env(k, v);
    }
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    if spec.tty.is_some() && !spec.new_session {
        return Err(Error::invalid("a controlling terminal requires a new session"));
    }
    let tty_c: Option<CString> = match &spec.tty {
        Some(p) => Some(CString::new(p.as_str()).map_err(|_| Error::invalid("NUL byte in tty path"))?),
        None => None,
    };
    let creds = spec.creds.clone();
    let new_session = spec.new_session;
    let umask_v = spec.umask & 0o777;
    let nnp = spec.no_new_privs;
    let pdeath = spec.die_with_parent;
    let parent = unsafe { ffi::getpid() };
    // SAFETY: `child_setup` only makes async-signal-safe calls and allocates
    // nothing, which is what a `pre_exec` hook running between fork and exec
    // requires. The hook is built outside the `unsafe` block so that block
    // contains only the `pre_exec` call itself.
    let hook = move || -> io::Result<()> { unsafe { child_setup(new_session, &tty_c, &map, &creds, &cwd_c, umask_v, nnp, pdeath, parent) } };
    unsafe {
        cmd.pre_exec(hook);
    }
    let child = cmd.spawn()?;
    drop(keep_alive);
    Ok(child)
}

// ---------------------------------------------------------------- VT, framebuffer, evdev

fn ioctl_val(fd: RawFd, req: c_ulong, arg: c_ulong, name: &'static str) -> Result<()> {
    if unsafe { ffi::ioctl(fd, req, arg) } < 0 {
        return Err(Error::sys(name));
    }
    Ok(())
}

/// Opens `/dev/tty0` and asks the kernel for a free (unused) virtual
/// terminal number via `VT_OPENQRY`.
pub fn vt_open_query() -> Result<u32> {
    let f = std::fs::OpenOptions::new().read(true).write(true).open("/dev/tty0")?;
    let mut n: c_int = 0;
    if unsafe { ffi::ioctl(f.as_raw_fd(), ffi::VT_OPENQRY, &mut n as *mut c_int) } < 0 {
        return Err(Error::sys("VT_OPENQRY"));
    }
    if n <= 0 {
        return Err(Error::display("kernel returned no free VT"));
    }
    Ok(n as u32)
}

/// `VT_GETSTATE`: which VT is currently on-screen.
pub fn vt_get_active(fd: RawFd) -> Result<u32> {
    let mut st = ffi::VtStat::default();
    if unsafe { ffi::ioctl(fd, ffi::VT_GETSTATE, &mut st as *mut ffi::VtStat) } < 0 {
        return Err(Error::sys("VT_GETSTATE"));
    }
    Ok(st.v_active as u32)
}

pub fn vt_activate(fd: RawFd, vt: u32) -> Result<()> {
    ioctl_val(fd, ffi::VT_ACTIVATE, vt as c_ulong, "VT_ACTIVATE")
}

pub fn vt_wait_active(fd: RawFd, vt: u32) -> Result<()> {
    ioctl_val(fd, ffi::VT_WAITACTIVE, vt as c_ulong, "VT_WAITACTIVE")
}

/// Locks (or unlocks) virtual-terminal switching so a text VT cannot be reached
/// from the login screen.
pub fn vt_lock_switch(fd: RawFd, lock: bool) -> Result<()> {
    let req = if lock { ffi::VT_LOCKSWITCH } else { ffi::VT_UNLOCKSWITCH };
    ioctl_val(fd, req, 1 as c_ulong, "VT_LOCKSWITCH")
}

/// `KDSETMODE`: graphics mode stops the console from drawing over the framebuffer.
pub fn kd_set_graphics(fd: RawFd, graphics: bool) -> Result<()> {
    ioctl_val(fd, ffi::KDSETMODE, if graphics { 1 } else { 0 }, "KDSETMODE")
}

pub fn kb_get_mode(fd: RawFd) -> Result<i32> {
    let mut m: c_int = 0;
    if unsafe { ffi::ioctl(fd, ffi::KDGKBMODE, &mut m as *mut c_int) } < 0 {
        return Err(Error::sys("KDGKBMODE"));
    }
    Ok(m)
}

/// `KDSKBMODE`: `K_OFF` (4) stops the kernel from echoing typed keys into the
/// console line discipline, `K_UNICODE` (3) restores normal behaviour.
pub fn kb_set_mode(fd: RawFd, mode: i32) -> Result<()> {
    ioctl_val(fd, ffi::KDSKBMODE, mode as c_ulong, "KDSKBMODE")
}

/// Exclusive access to an evdev device (`EVIOCGRAB`).
pub fn evdev_grab(fd: RawFd, grab: bool) -> Result<()> {
    ioctl_val(fd, ffi::EVIOCGRAB, if grab { 1 } else { 0 }, "EVIOCGRAB")
}

/// A single evdev event (`struct input_event`, time fields discarded).
#[derive(Clone, Copy, Debug)]
pub struct RawInputEvent {
    pub type_: u16,
    pub code: u16,
    pub value: i32,
}

pub const EV_SYN: u16 = ffi::EV_SYN;
pub const EV_KEY: u16 = ffi::EV_KEY;
pub const EV_REL: u16 = ffi::EV_REL;

/// Reads whatever whole `input_event` records are currently available on
/// `fd` (a non-blocking evdev node). A short/partial trailing read is kept
/// out of the result and left for next time.
pub fn evdev_read(fd: RawFd) -> Vec<RawInputEvent> {
    const SZ: usize = std::mem::size_of::<ffi::InputEvent>();
    let mut buf = [0u8; SZ * 32];
    let mut out = Vec::new();
    loop {
        let n = unsafe { libc_read(fd, buf.as_mut_ptr() as *mut c_void, buf.len()) };
        if n <= 0 {
            break;
        }
        let n = n as usize;
        let mut off = 0;
        while off + SZ <= n {
            let mut ev: ffi::InputEvent = unsafe { std::mem::zeroed() };
            unsafe {
                std::ptr::copy_nonoverlapping(buf[off..].as_ptr(), &mut ev as *mut _ as *mut u8, SZ);
            }
            out.push(RawInputEvent { type_: ev.type_, code: ev.code, value: ev.value });
            off += SZ;
        }
        if n < buf.len() {
            break;
        }
        if out.len() > 4096 {
            break;
        }
    }
    out
}

extern "C" {
    #[link_name = "read"]
    fn libc_read(fd: c_int, buf: *mut c_void, count: usize) -> isize;
}

/// Human-readable device name via `EVIOCGNAME`, best-effort only (used for
/// logs/config matching, never for security decisions).
pub fn evdev_name(fd: RawFd) -> Option<String> {
    let mut buf = [0u8; 256];
    let req = ffi::eviocgname(buf.len());
    let n = unsafe { ffi::ioctl(fd, req, buf.as_mut_ptr()) };
    if n <= 0 {
        return None;
    }
    let n = (n as usize).min(buf.len());
    let end = buf[..n].iter().position(|&b| b == 0).unwrap_or(n);
    Some(String::from_utf8_lossy(&buf[..end]).into_owned())
}

fn get_bits(fd: RawFd, ev: u16, nbits: usize) -> Vec<u64> {
    let nwords = nbits.div_ceil(64);
    let mut words = vec![0u64; nwords];
    let req = ffi::eviocgbit(ev, nwords * 8);
    unsafe {
        ffi::ioctl(fd, req, words.as_mut_ptr());
    }
    words
}

fn bit_set(words: &[u64], bit: u16) -> bool {
    let (w, b) = (bit as usize / 64, bit as usize % 64);
    words.get(w).map(|word| (word >> b) & 1 != 0).unwrap_or(false)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DeviceCaps {
    pub has_keys: bool,
    pub has_letter_keys: bool,
    pub has_rel: bool,
    pub has_mouse_buttons: bool,
}

/// Classifies an already-open evdev node purely by reported capability bits
/// (`EVIOCGBIT`), never by name, so a keyboard/mouse combo device or an
/// oddly-named device is still handled correctly.
pub fn evdev_capabilities(fd: RawFd) -> DeviceCaps {
    let ev_bits = get_bits(fd, 0, 32);
    let has_key_type = bit_set(&ev_bits, EV_KEY);
    let has_rel_type = bit_set(&ev_bits, EV_REL);
    let key_bits = if has_key_type { get_bits(fd, EV_KEY, 768) } else { Vec::new() };
    let has_letter_keys = bit_set(&key_bits, ffi::KEY_A) && bit_set(&key_bits, ffi::KEY_Z);
    let has_mouse_buttons = bit_set(&key_bits, ffi::BTN_LEFT);
    DeviceCaps {
        has_keys: has_key_type,
        has_letter_keys,
        has_rel: has_rel_type,
        has_mouse_buttons,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FbInfo {
    pub width: u32,
    pub height: u32,
    pub bpp: u32,
    pub line_length: u32,
    pub smem_len: u32,
    /// `(offset, length)` of each colour channel within a pixel, in bits.
    pub red: (u32, u32),
    pub green: (u32, u32),
    pub blue: (u32, u32),
    pub width_mm: u32,
    pub height_mm: u32,
}

pub fn fb_info(fd: RawFd) -> Result<FbInfo> {
    let mut var = ffi::FbVarScreeninfo::default();
    let mut fix: ffi::FbFixScreeninfo = unsafe { std::mem::zeroed() };
    unsafe {
        if ffi::ioctl(fd, ffi::FBIOGET_VSCREENINFO, &mut var as *mut ffi::FbVarScreeninfo) < 0 {
            return Err(Error::sys("FBIOGET_VSCREENINFO"));
        }
        if ffi::ioctl(fd, ffi::FBIOGET_FSCREENINFO, &mut fix as *mut ffi::FbFixScreeninfo) < 0 {
            return Err(Error::sys("FBIOGET_FSCREENINFO"));
        }
    }
    Ok(FbInfo {
        width: var.xres,
        height: var.yres,
        bpp: var.bits_per_pixel,
        line_length: fix.line_length,
        smem_len: fix.smem_len,
        red: (var.red.offset, var.red.length),
        green: (var.green.offset, var.green.length),
        blue: (var.blue.offset, var.blue.length),
        width_mm: if var.width < 100000 { var.width } else { 0 },
        height_mm: if var.height < 100000 { var.height } else { 0 },
    })
}

/// A shared writable mapping of the framebuffer device.
pub struct FbMap {
    ptr: *mut u8,
    len: usize,
}

unsafe impl Send for FbMap {}

impl FbMap {
    pub fn new(fd: RawFd, len: usize) -> Result<FbMap> {
        if len == 0 || len > (1usize << 30) {
            return Err(Error::display("implausible framebuffer size"));
        }
        let p = unsafe {
            ffi::mmap(
                std::ptr::null_mut(),
                len,
                ffi::PROT_READ | ffi::PROT_WRITE,
                ffi::MAP_SHARED,
                fd,
                0,
            )
        };
        if p as isize == -1 || p.is_null() {
            return Err(Error::sys("mmap(framebuffer)"));
        }
        Ok(FbMap { ptr: p as *mut u8, len })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl Drop for FbMap {
    fn drop(&mut self) {
        unsafe {
            ffi::munmap(self.ptr as *mut c_void, self.len);
        }
    }
}

// ---------------------------------------------------------------- terminal echo, power

/// Restores terminal echo when dropped.
pub struct EchoGuard {
    fd: RawFd,
    saved: [u32; 15],
}

/// Turns terminal echo off on `fd` (a tty). Used for password prompts.
pub fn disable_echo(fd: RawFd) -> Result<EchoGuard> {
    let mut t = [0u32; 15];
    if unsafe { ffi::tcgetattr(fd, t.as_mut_ptr() as *mut c_void) } != 0 {
        return Err(Error::sys("tcgetattr"));
    }
    let saved = t;
    t[ffi::LFLAG_INDEX] &= !ffi::ECHO;
    if unsafe { ffi::tcsetattr(fd, ffi::TCSANOW, t.as_ptr() as *const c_void) } != 0 {
        return Err(Error::sys("tcsetattr"));
    }
    Ok(EchoGuard { fd, saved })
}

impl Drop for EchoGuard {
    fn drop(&mut self) {
        unsafe {
            ffi::tcsetattr(self.fd, ffi::TCSANOW, self.saved.as_ptr() as *const c_void);
        }
    }
}

pub fn sync_disks() {
    unsafe { ffi::sync() }
}

/// `chown(2)`. Used only to fix ownership of `XDG_RUNTIME_DIR` right after
/// creating it as root, before the session ever runs.
/// Takes ownership of a file descriptor this process inherited at a known,
/// pre-arranged number (one the parent `dup2`'d into place before `exec`, as
/// [`spawn`] does for the greeter — see docs/architecture.md's process
/// model). The safety requirement `from_raw_fd` carries — that `fd` is
/// currently open and not otherwise owned — is exactly what the parent's
/// `spawn` call already guarantees for every fd number it documents to the
/// child; this wrapper exists so that guarantee only has to be reasoned
/// about once, here, rather than at every call site outside `sys` (which
/// `#![deny(unsafe_code)]` keeps `unsafe`-free entirely).
pub fn adopt_fd_as_file(fd: RawFd) -> File {
    unsafe { File::from_raw_fd(fd) }
}

/// As [`adopt_fd_as_file`], for a fd known to be the daemon end of a
/// `socketpair(2)` (the greeter's IPC link).
pub fn adopt_fd_as_unix_stream(fd: RawFd) -> std::os::unix::net::UnixStream {
    unsafe { std::os::unix::net::UnixStream::from_raw_fd(fd) }
}

/// Closes a raw fd this process owns and no typed handle tracks (e.g. our
/// copy of a socketpair end that `spawn` has already duplicated into a
/// child). The standard safe-looking idiom: wrap it in an `OwnedFd` and let
/// that drop.
pub fn close_fd(fd: RawFd) {
    if fd >= 0 {
        drop(unsafe { OwnedFd::from_raw_fd(fd) });
    }
}

pub fn chown_path(path: &std::path::Path, uid: u32, gid: u32) -> Result<()> {
    let c = CString::new(path.as_os_str().as_encoded_bytes()).map_err(|_| Error::invalid("NUL byte in path"))?;
    if unsafe { ffi::chown(c.as_ptr(), uid, gid) } != 0 {
        return Err(Error::sys("chown"));
    }
    Ok(())
}

pub fn reboot_now() -> Result<()> {
    sync_disks();
    if unsafe { ffi::reboot(ffi::RB_AUTOBOOT) } != 0 {
        return Err(Error::sys("reboot"));
    }
    Ok(())
}

pub fn poweroff_now() -> Result<()> {
    sync_disks();
    if unsafe { ffi::reboot(ffi::RB_POWER_OFF) } != 0 {
        return Err(Error::sys("reboot(POWER_OFF)"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zeroize_clears_bytes() {
        let mut b = vec![1u8, 2, 3, 4];
        zeroize(&mut b);
        assert_eq!(b, vec![0, 0, 0, 0]);
    }

    #[test]
    fn zeroize_vec_clears_and_empties() {
        let mut v = Vec::with_capacity(16);
        v.extend_from_slice(b"secret");
        zeroize_vec(&mut v);
        assert!(v.is_empty());
    }

    #[test]
    fn uid_lookup_for_root_works() {
        let e = getpwuid(0).unwrap().expect("root must exist");
        assert_eq!(e.uid, 0);
        assert_eq!(e.name, "root");
    }

    #[test]
    fn unknown_user_is_none() {
        assert!(getpwnam("definitely-not-a-user-xyz").unwrap().is_none());
    }

    #[test]
    fn peer_credentials_of_socketpair_are_ours() {
        use std::os::unix::net::UnixStream;
        let (a, _b) = UnixStream::pair().unwrap();
        let c = peer_credentials(a.as_raw_fd()).unwrap();
        assert_eq!(c.uid, getuid());
        assert_eq!(c.pid, getpid());
    }

    #[test]
    fn spawn_runs_a_child_with_clean_env() {
        let mut spec = SpawnSpec::new("/bin/sh");
        spec.args = vec!["-c".into(), "test -z \"$HOME\" && exit 0; exit 3".into()];
        spec.env = vec![("PATH".into(), "/usr/bin:/bin".into())];
        let mut child = spawn(&spec).unwrap();
        assert!(child.wait().unwrap().success());
    }
}
