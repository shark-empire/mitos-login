/// Mirrors the shape of `mitos-session`'s own `AuthOutcome` (see that
/// project's `ipc/messages.rs`) so the two daemons speak a consistent
/// vocabulary even though they run entirely separate wire protocols: this one
/// for the initial greeter, that one for the in-session lock screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthOutcome {
    Success,
    Failure { attempts_remaining: Option<u32> },
    LockedOut { retry_after_secs: u64 },
    Error(String),
}
