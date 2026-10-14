//! Minimal Linux-PAM binding, loaded with `dlopen` so the crate has no
//! build-time dependency on libpam and can fall back to the shadow backend on
//! systems without it.
use super::{ffi, zeroize, Library};
use crate::errors::{Error, Result};
use crate::security::secret::Secret;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::sync::Arc;

pub const PAM_SUCCESS: c_int = 0;
pub const PAM_BUF_ERR: c_int = 5;
pub const PAM_PERM_DENIED: c_int = 6;
pub const PAM_AUTH_ERR: c_int = 7;
pub const PAM_CRED_INSUFFICIENT: c_int = 8;
pub const PAM_AUTHINFO_UNAVAIL: c_int = 9;
pub const PAM_USER_UNKNOWN: c_int = 10;
pub const PAM_MAXTRIES: c_int = 11;
pub const PAM_NEW_AUTHTOK_REQD: c_int = 12;
pub const PAM_ACCT_EXPIRED: c_int = 13;
pub const PAM_CONV_ERR: c_int = 19;

pub const PAM_SILENT: c_int = 0x8000;
pub const PAM_DISALLOW_NULL_AUTHTOK: c_int = 0x0001;
pub const PAM_ESTABLISH_CRED: c_int = 0x0002;
pub const PAM_DELETE_CRED: c_int = 0x0004;
pub const PAM_CHANGE_EXPIRED_AUTHTOK: c_int = 0x0020;

pub const PAM_TTY: c_int = 3;
pub const PAM_RHOST: c_int = 4;

const PAM_PROMPT_ECHO_OFF: c_int = 1;
const PAM_PROMPT_ECHO_ON: c_int = 2;
const PAM_ERROR_MSG: c_int = 3;
const PAM_TEXT_INFO: c_int = 4;

/// Receives PAM's questions and messages.
pub trait PamConv: Send {
    /// A prompt from a PAM module. `None` aborts the conversation.
    fn prompt(&mut self, echo: bool, text: &str) -> Option<Secret>;
    fn info(&mut self, text: &str);
    fn error(&mut self, text: &str);
}

#[repr(C)]
struct PamMessage {
    msg_style: c_int,
    msg: *const c_char,
}

#[repr(C)]
struct PamResponse {
    resp: *mut c_char,
    resp_retcode: c_int,
}

type ConvFn = unsafe extern "C" fn(c_int, *mut *const PamMessage, *mut *mut PamResponse, *mut c_void) -> c_int;

#[repr(C)]
struct PamConvC {
    conv: ConvFn,
    appdata_ptr: *mut c_void,
}

type StartFn = unsafe extern "C" fn(*const c_char, *const c_char, *const PamConvC, *mut *mut c_void) -> c_int;
type EndFn = unsafe extern "C" fn(*mut c_void, c_int) -> c_int;
type FlagFn = unsafe extern "C" fn(*mut c_void, c_int) -> c_int;
type EnvFn = unsafe extern "C" fn(*mut c_void) -> *mut *mut c_char;
type SetItemFn = unsafe extern "C" fn(*mut c_void, c_int, *const c_void) -> c_int;
type StrErrFn = unsafe extern "C" fn(*mut c_void, c_int) -> *const c_char;

/// The resolved PAM entry points.
pub struct PamApi {
    start: StartFn,
    end: EndFn,
    authenticate: FlagFn,
    acct_mgmt: FlagFn,
    setcred: FlagFn,
    open_session: FlagFn,
    close_session: FlagFn,
    chauthtok: FlagFn,
    getenvlist: EnvFn,
    set_item: SetItemFn,
    strerror: StrErrFn,
    _lib: Library,
}

unsafe impl Send for PamApi {}
unsafe impl Sync for PamApi {}

impl PamApi {
    /// Loads libpam. `explicit` may name an absolute library path; otherwise
    /// the standard root-owned locations are tried first.
    pub fn load(explicit: &str) -> Result<PamApi> {
        let candidates: Vec<String> = if explicit.is_empty() {
            super::system_library_candidates("libpam.so.0")
        } else {
            vec![explicit.to_string()]
        };
        let refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();
        let lib = Library::open_any(&refs)?;
        unsafe {
            let api = PamApi {
                start: std::mem::transmute::<*mut c_void, StartFn>(lib.symbol("pam_start")?),
                end: std::mem::transmute::<*mut c_void, EndFn>(lib.symbol("pam_end")?),
                authenticate: std::mem::transmute::<*mut c_void, FlagFn>(lib.symbol("pam_authenticate")?),
                acct_mgmt: std::mem::transmute::<*mut c_void, FlagFn>(lib.symbol("pam_acct_mgmt")?),
                setcred: std::mem::transmute::<*mut c_void, FlagFn>(lib.symbol("pam_setcred")?),
                open_session: std::mem::transmute::<*mut c_void, FlagFn>(lib.symbol("pam_open_session")?),
                close_session: std::mem::transmute::<*mut c_void, FlagFn>(lib.symbol("pam_close_session")?),
                chauthtok: std::mem::transmute::<*mut c_void, FlagFn>(lib.symbol("pam_chauthtok")?),
                getenvlist: std::mem::transmute::<*mut c_void, EnvFn>(lib.symbol("pam_getenvlist")?),
                set_item: std::mem::transmute::<*mut c_void, SetItemFn>(lib.symbol("pam_set_item")?),
                strerror: std::mem::transmute::<*mut c_void, StrErrFn>(lib.symbol("pam_strerror")?),
                _lib: lib,
            };
            Ok(api)
        }
    }
}

struct ConvState {
    conv: Box<dyn PamConv>,
}

/// One PAM transaction (`pam_start` .. `pam_end`).
pub struct PamSession {
    api: Arc<PamApi>,
    handle: *mut c_void,
    _state: Box<ConvState>,
    _cconv: Box<PamConvC>,
    last: c_int,
}

unsafe impl Send for PamSession {}

unsafe fn free_replies(replies: *mut PamResponse, upto: usize) {
    for j in 0..upto {
        let r = (*replies.add(j)).resp;
        if !r.is_null() {
            let len = CStr::from_ptr(r).to_bytes().len();
            zeroize(std::slice::from_raw_parts_mut(r as *mut u8, len));
            ffi::free(r as *mut c_void);
        }
    }
    ffi::free(replies as *mut c_void);
}

unsafe fn conv_inner(
    num_msg: c_int,
    msg: *mut *const PamMessage,
    resp: *mut *mut PamResponse,
    appdata: *mut c_void,
) -> c_int {
    if num_msg <= 0 || num_msg > 32 || msg.is_null() || resp.is_null() || appdata.is_null() {
        return PAM_CONV_ERR;
    }
    let n = num_msg as usize;
    let state = &mut *(appdata as *mut ConvState);
    let replies = ffi::calloc(n, std::mem::size_of::<PamResponse>()) as *mut PamResponse;
    if replies.is_null() {
        return PAM_BUF_ERR;
    }
    for i in 0..n {
        let m = *msg.add(i);
        if m.is_null() {
            free_replies(replies, i);
            return PAM_CONV_ERR;
        }
        let style = (*m).msg_style;
        let text = if (*m).msg.is_null() {
            String::new()
        } else {
            CStr::from_ptr((*m).msg).to_string_lossy().into_owned()
        };
        match style {
            PAM_PROMPT_ECHO_OFF | PAM_PROMPT_ECHO_ON => {
                let echo = style == PAM_PROMPT_ECHO_ON;
                match state.conv.prompt(echo, &text) {
                    Some(secret) => {
                        let bytes = secret.expose();
                        let p = ffi::malloc(bytes.len() + 1) as *mut u8;
                        if p.is_null() {
                            free_replies(replies, i);
                            return PAM_BUF_ERR;
                        }
                        std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
                        *p.add(bytes.len()) = 0;
                        (*replies.add(i)).resp = p as *mut c_char;
                    }
                    None => {
                        free_replies(replies, i);
                        return PAM_CONV_ERR;
                    }
                }
            }
            PAM_ERROR_MSG => state.conv.error(&text),
            PAM_TEXT_INFO => state.conv.info(&text),
            _ => {
                free_replies(replies, i);
                return PAM_CONV_ERR;
            }
        }
    }
    *resp = replies;
    PAM_SUCCESS
}

/// The C callback. A panic must never unwind into libpam.
unsafe extern "C" fn conv_trampoline(
    num_msg: c_int,
    msg: *mut *const PamMessage,
    resp: *mut *mut PamResponse,
    appdata: *mut c_void,
) -> c_int {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| conv_inner(num_msg, msg, resp, appdata))) {
        Ok(rc) => rc,
        Err(_) => PAM_CONV_ERR,
    }
}

impl PamSession {
    pub fn start(api: &Arc<PamApi>, service: &str, user: &str, conv: Box<dyn PamConv>) -> Result<PamSession> {
        let svc = CString::new(service).map_err(|_| Error::invalid("NUL byte in PAM service"))?;
        let usr = CString::new(user).map_err(|_| Error::invalid("NUL byte in user name"))?;
        let mut state = Box::new(ConvState { conv });
        let cconv = Box::new(PamConvC {
            conv: conv_trampoline,
            appdata_ptr: &mut *state as *mut ConvState as *mut c_void,
        });
        let mut handle: *mut c_void = std::ptr::null_mut();
        let rc = unsafe { (api.start)(svc.as_ptr(), usr.as_ptr(), &*cconv as *const PamConvC, &mut handle) };
        if rc != PAM_SUCCESS || handle.is_null() {
            return Err(Error::Auth(format!("pam_start failed ({})", rc)));
        }
        Ok(PamSession { api: api.clone(), handle, _state: state, _cconv: cconv, last: PAM_SUCCESS })
    }

    fn call(&mut self, f: FlagFn, flags: c_int) -> c_int {
        let rc = unsafe { f(self.handle, flags) };
        self.last = rc;
        rc
    }

    pub fn authenticate(&mut self, flags: c_int) -> c_int {
        let f = self.api.authenticate;
        self.call(f, flags)
    }

    pub fn acct_mgmt(&mut self, flags: c_int) -> c_int {
        let f = self.api.acct_mgmt;
        self.call(f, flags)
    }

    pub fn setcred(&mut self, flags: c_int) -> c_int {
        let f = self.api.setcred;
        self.call(f, flags)
    }

    pub fn open_session(&mut self, flags: c_int) -> c_int {
        let f = self.api.open_session;
        self.call(f, flags)
    }

    pub fn close_session(&mut self, flags: c_int) -> c_int {
        let f = self.api.close_session;
        self.call(f, flags)
    }

    pub fn chauthtok(&mut self, flags: c_int) -> c_int {
        let f = self.api.chauthtok;
        self.call(f, flags)
    }

    /// Sets a string item such as `PAM_TTY`.
    pub fn set_item_str(&mut self, item: c_int, value: &str) -> bool {
        let c = match CString::new(value) {
            Ok(c) => c,
            Err(_) => return false,
        };
        let rc = unsafe { (self.api.set_item)(self.handle, item, c.as_ptr() as *const c_void) };
        rc == PAM_SUCCESS
    }

    /// The environment PAM modules want the session to have.
    pub fn env(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        unsafe {
            let list = (self.api.getenvlist)(self.handle);
            if list.is_null() {
                return out;
            }
            let mut i: isize = 0;
            loop {
                let p = *list.offset(i);
                if p.is_null() || i > 4096 {
                    break;
                }
                let s = CStr::from_ptr(p).to_string_lossy().into_owned();
                if let Some(eq) = s.find('=') {
                    out.push((s[..eq].to_string(), s[eq + 1..].to_string()));
                }
                ffi::free(p as *mut c_void);
                i += 1;
            }
            ffi::free(list as *mut c_void);
        }
        out
    }

    pub fn strerror(&self, code: c_int) -> String {
        unsafe {
            let p = (self.api.strerror)(self.handle, code);
            if p.is_null() {
                String::from("unknown PAM error")
            } else {
                CStr::from_ptr(p).to_string_lossy().into_owned()
            }
        }
    }
}

impl Drop for PamSession {
    fn drop(&mut self) {
        unsafe {
            (self.api.end)(self.handle, self.last);
        }
    }
}
