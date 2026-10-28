//! Authenticating a user: PAM first, with a direct-`crypt(3)`-against-`/etc/shadow`
//! fallback for systems with no usable PAM stack. Runs only inside the root
//! daemon — the greeter never touches a password beyond relaying the bytes
//! the person typed.
pub mod client;
pub mod errors;
pub mod methods;
pub mod request;
pub mod response;
pub mod shadow;

pub use methods::AuthMethod;
pub use request::AuthRequest;
pub use response::AuthOutcome;
