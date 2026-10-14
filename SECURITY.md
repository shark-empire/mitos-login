# Security policy

See `docs/security.md` for the full threat model, the credential-handling,
lockout, audit-log, and privilege-drop design, and what this project
deliberately does not attempt.

## Reporting a vulnerability

This is a sample/personal project without a dedicated security contact or
disclosure program at this time. If you find an issue, open it as you would
any other bug -- there is no embargo process here to route around.

## Scope notes for anyone auditing this

- All raw FFI (`src/sys/ffi.rs`) is cross-checked against real system
  headers by `scripts/check-ffi.sh` -- re-run it after any change there,
  on the actual target architecture, before trusting a struct layout.
- `cargo build && cargo test` has not been run in this environment (see
  `README.md`'s Status section). Treat this as unaudited, pre-first-compile
  code, not a hardened release, until that changes.
- The daemon runs as plain root for its entire lifetime with no sandboxing
  (`docs/security.md`'s "What this project does not attempt"). Don't assume
  a bug here is contained by anything other than the privilege-drop logic
  in `src/sys/mod.rs::spawn` itself.
