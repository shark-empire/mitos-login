/// Which credential the person is presenting. PIN reuses the same PAM/shadow
/// path under a distinct PAM service name (`mitos-login-pin`), so a site can
/// configure it (e.g. only for accounts with a PIN set) without touching code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AuthMethod {
    Password,
    Pin,
}

impl AuthMethod {
    pub fn pam_service(&self) -> &'static str {
        match self {
            AuthMethod::Password => "mitos-login",
            AuthMethod::Pin => "mitos-login-pin",
        }
    }

    pub fn prompt_label(&self) -> &'static str {
        match self {
            AuthMethod::Password => "Password",
            AuthMethod::Pin => "PIN",
        }
    }
}
