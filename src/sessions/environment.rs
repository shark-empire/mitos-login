//! Building the child session's environment from scratch. Nothing is
//! inherited from the daemon's own environment — a login process is exactly
//! where an inherited stray variable (an attacker-controlled `LD_PRELOAD`,
//! a leftover secret) would be most dangerous.
use crate::users::User;

pub struct EnvBuilder {
    vars: Vec<(String, String)>,
}

impl EnvBuilder {
    pub fn new() -> EnvBuilder {
        EnvBuilder { vars: Vec::new() }
    }

    pub fn set(mut self, key: &str, value: impl Into<String>) -> EnvBuilder {
        self.vars.retain(|(k, _)| k != key);
        self.vars.push((key.to_string(), value.into()));
        self
    }

    /// PAM-supplied variables never override the base set this builder
    /// already computed (`HOME`, `USER`, ...) — a PAM module has no business
    /// changing who the session runs as.
    pub fn extend_without_override(mut self, pairs: Vec<(String, String)>) -> EnvBuilder {
        for (k, v) in pairs {
            if !matches!(k.as_str(), "HOME" | "USER" | "LOGNAME" | "SHELL" | "UID") && !self.vars.iter().any(|(ek, _)| ek == &k) {
                self.vars.push((k, v));
            }
        }
        self
    }

    pub fn build(self) -> Vec<(String, String)> {
        self.vars
    }
}

impl Default for EnvBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn base_environment(
    user: &User,
    session_id: u32,
    seat: &str,
    runtime_dir: &str,
    tty: &str,
    lang: &str,
    session_type: &str,
) -> EnvBuilder {
    let shell = if user.shell.is_empty() { "/bin/sh".to_string() } else { user.shell.clone() };
    EnvBuilder::new()
        .set("HOME", user.home.clone())
        .set("USER", user.name.clone())
        .set("LOGNAME", user.name.clone())
        .set("SHELL", shell)
        .set("PATH", "/usr/local/bin:/usr/bin:/bin")
        .set("XDG_SESSION_ID", session_id.to_string())
        .set("XDG_SEAT", seat.to_string())
        .set("XDG_RUNTIME_DIR", runtime_dir.to_string())
        .set("XDG_SESSION_TYPE", session_type.to_string())
        .set("XDG_SESSION_CLASS", "user")
        .set("LANG", lang.to_string())
        .set("TERM", "linux")
        .set("MITOS_TTY", tty.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn later_set_overrides_earlier() {
        let env = EnvBuilder::new().set("A", "1").set("A", "2").build();
        assert_eq!(env, vec![("A".to_string(), "2".to_string())]);
    }

    #[test]
    fn pam_vars_cannot_override_identity() {
        let env = EnvBuilder::new()
            .set("HOME", "/home/alice")
            .extend_without_override(vec![("HOME".to_string(), "/tmp/evil".to_string()), ("SSH_AUTH_SOCK".to_string(), "/tmp/sock".to_string())])
            .build();
        let home = env.iter().find(|(k, _)| k == "HOME").unwrap();
        assert_eq!(home.1, "/home/alice");
        assert!(env.iter().any(|(k, _)| k == "SSH_AUTH_SOCK"));
    }
}
