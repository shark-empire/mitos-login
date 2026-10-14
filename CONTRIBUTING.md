# Contributing

## Before changing `src/sys/ffi.rs`

Every constant and struct layout there must match the real system headers.
Run `sh scripts/check-ffi.sh` (needs `gcc` and `python3`) after any change
and before opening a PR -- it compiles a small C probe
(`scripts/ffi_check.c`) against your actual headers and diffs the results
against what the Rust side declares.

## Style

- Zero third-party `[dependencies]` in `Cargo.toml` is a deliberate
  project choice (see the README), not an oversight -- this crate runs as
  root next to the login screen, so every dependency would be part of the
  trusted base. New functionality should stay within the standard library
  plus `src/sys`'s existing FFI, not add a crate.
- All `unsafe` lives in `src/sys/` (`#![deny(unsafe_code)]` everywhere
  else in `src/lib.rs`). If you need a new raw syscall/ioctl, add it there
  with the same verification discipline as the rest of that module.
- Favor returning a clear `Error` over `panic!`/`.unwrap()` anywhere input
  could originate from the greeter, a config file, or `/etc/passwd`
  /`/etc/shadow` -- the daemon runs as root and must not crash on
  malformed input from any of those.

## Tests

Most modules carry their own `#[cfg(test)]` unit tests; `tests/` holds
integration tests that only exercise the crate's public API. Run both with
`cargo test`.
