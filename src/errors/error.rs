use std::fmt;
use std::io;

/// The single error type used across mitos-login.
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    /// A failed system call with its errno.
    Sys { call: &'static str, errno: i32 },
    Config(String),
    Protocol(String),
    Auth(String),
    Session(String),
    Security(String),
    Display(String),
    Invalid(String),
    Timeout(&'static str),
    Unsupported(&'static str),
    /// A shared library or symbol could not be loaded.
    Load(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn config(m: impl Into<String>) -> Error {
        Error::Config(m.into())
    }
    pub fn protocol(m: impl Into<String>) -> Error {
        Error::Protocol(m.into())
    }
    pub fn auth(m: impl Into<String>) -> Error {
        Error::Auth(m.into())
    }
    pub fn session(m: impl Into<String>) -> Error {
        Error::Session(m.into())
    }
    pub fn security(m: impl Into<String>) -> Error {
        Error::Security(m.into())
    }
    pub fn display(m: impl Into<String>) -> Error {
        Error::Display(m.into())
    }
    pub fn invalid(m: impl Into<String>) -> Error {
        Error::Invalid(m.into())
    }
    /// Captures `errno` of the call that just failed. Call it immediately.
    pub fn sys(call: &'static str) -> Error {
        Error::Sys {
            call,
            errno: io::Error::last_os_error().raw_os_error().unwrap_or(0),
        }
    }

    /// A short message that is safe to show on the login screen. Never
    /// contains paths, usernames, errno values or other internals.
    pub fn public_message(&self) -> &'static str {
        match self {
            Error::Auth(_) => "Authentication failed",
            Error::Timeout(_) => "The request timed out",
            Error::Invalid(_) | Error::Protocol(_) => "Invalid request",
            Error::Security(_) => "Not permitted",
            _ => "Internal error",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Sys { call, errno } => {
                write!(f, "{} failed: {}", call, io::Error::from_raw_os_error(*errno))
            }
            Error::Config(m) => write!(f, "configuration error: {}", m),
            Error::Protocol(m) => write!(f, "protocol error: {}", m),
            Error::Auth(m) => write!(f, "authentication error: {}", m),
            Error::Session(m) => write!(f, "session error: {}", m),
            Error::Security(m) => write!(f, "security policy: {}", m),
            Error::Display(m) => write!(f, "display error: {}", m),
            Error::Invalid(m) => write!(f, "invalid input: {}", m),
            Error::Timeout(w) => write!(f, "timed out: {}", w),
            Error::Unsupported(w) => write!(f, "unsupported: {}", w),
            Error::Load(m) => write!(f, "cannot load library: {}", m),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_messages_do_not_leak_details() {
        let e = Error::Auth("user alice: pam_unix says no".into());
        assert_eq!(e.public_message(), "Authentication failed");
        assert!(!e.public_message().contains("alice"));
        assert_eq!(Error::config("x").public_message(), "Internal error");
    }

    #[test]
    fn display_includes_context() {
        let e = Error::Sys { call: "poll", errno: 4 };
        assert!(e.to_string().starts_with("poll failed"));
    }
}
