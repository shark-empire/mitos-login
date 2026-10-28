use crate::security::Secret;
use super::methods::AuthMethod;

/// One authentication attempt handed from the IPC layer to [`super::client`].
pub struct AuthRequest {
    pub user: String,
    pub method: AuthMethod,
    pub secret: Secret,
    /// The tty/seat name PAM should record (`PAM_TTY`), e.g. `"tty1"`.
    pub tty: String,
}
