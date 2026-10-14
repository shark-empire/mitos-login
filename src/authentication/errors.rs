use crate::errors::Error;

/// Maps a raw PAM return code to the crate-wide [`Error`] type with a message
/// that never reaches the greeter directly — [`Error::public_message`] is
/// what the UI actually shows.
pub fn pam_error(code: i32, detail: &str) -> Error {
    Error::Auth(format!("{} (code {})", detail, code))
}
