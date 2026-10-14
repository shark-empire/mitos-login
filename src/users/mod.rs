//! Who can be shown on the greeter's user list and looked up by name.
//! Single-user lookups (needed to authenticate *any* valid account, including
//! one NSS provides that isn't a plain `/etc/passwd` line) go through libc's
//! `getpwnam_r`/`getpwuid_r` ([`crate::sys`]); the visible list is built by
//! directly reading `/etc/passwd`, matching this project's existing
//! convention (see mitos-utils' account tools) of treating that file's
//! well-defined format as safe to parse by hand rather than depend on
//! `getpwent`.
pub mod account;
pub mod avatar;
pub mod enumeration;
pub mod manager;
pub mod user;

pub use manager::UserManager;
pub use user::User;
