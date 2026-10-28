//! Security policy: secret handling, hashing, lockout, rate limiting, audit,
//! and file-trust checks. Nothing in this module is exposed to the greeter
//! process directly — it is used only from inside the root daemon.
pub mod audit;
pub mod hash;
pub mod lockout;
pub mod permissions;
pub mod policy;
pub mod rate_limit;
pub mod secret;
pub mod timeout;

pub use secret::Secret;

use crate::errors::Result;
use std::io::Read;

/// Reads `n` bytes from `/dev/urandom`. Since Linux 3.17 this is
/// cryptographically equivalent to `getrandom(2)` once the kernel's CSPRNG is
/// seeded, which it always is by the time a login daemon is running well
/// after boot; using the file interface avoids an extra raw syscall binding.
pub fn urandom(n: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; n];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut buf)?;
    Ok(buf)
}

