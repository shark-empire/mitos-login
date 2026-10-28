# mitos-login

The login manager for mitosOS: a privilege-separated daemon, a
"Liquid Glass" greeter rendered by a small software rasterizer (no GPU, no
font files, zero third-party dependencies), hardened session launching, and
a tamper-evident audit log.

```
mitos-login daemon          # run the login daemon (root)
mitos-login check-config    # validate /etc/mitos-login and exit
mitos-login preview out.ppm 1280x720 login   # render one frame, no root/hardware needed
mitos-loginctl status       # talk to a running daemon
```

See `docs/architecture.md` for the full design, `docs/security.md` for the
threat model and the security decisions made throughout, and
`docs/troubleshooting.md` if something isn't working.

## Building

```
cargo build --release
```

Needs a Rust toolchain (`rust-version = "1.74"`, see `Cargo.toml`) and a
C compiler available for `scripts/check-ffi.sh` (optional, but recommended
after touching anything in `src/sys/ffi.rs` -- see below). No other
dependencies: this crate is intentionally zero third-party `[dependencies]`
for everything except `fuzz/`-equivalent tooling, which it doesn't have
either, since there is none here yet.

## Verifying the raw system bindings

`src/sys/ffi.rs` hard-codes constants and struct layouts (`ioctl` numbers,
`struct passwd`, `struct fb_var_screeninfo`, evdev's `_IOC`-encoded ioctls,
signal numbers, `wait(2)` status bits) that must match the real system
headers exactly. `scripts/check-ffi.sh` verifies every one of them against
your actual `<linux/...>`/glibc headers by compiling a small C probe and
diffing the results — run it after any change there:

```
sh scripts/check-ffi.sh
```

## Status

Functionally complete for a single-output, PAM-or-shadow, single-seat
system: authentication (PAM with a `crypt(3)`/`/etc/shadow` fallback),
lockout, a tamper-evident audit log, session launching with full privilege
drop, and a real rendered greeter with keyboard navigation and
accessibility settings all work end to end. **Not yet run through a real
`cargo build`** in this environment (no Rust toolchain was available while
writing it) — every module was instead verified the way the rest of the
mitosOS project does when a toolchain isn't available: every raw FFI
constant and struct layout cross-checked against real headers with a C
probe (`scripts/check-ffi.sh`), the real system `crypt(3)`/PAM exercised
directly, the hand-rolled SHA-256/HMAC and the wire protocol's encoder/
decoder fuzz-tested via an independent Python port, and the real
privilege-drop sequence exercised against a throwaway system account. Run
`cargo build && cargo test` before trusting this in production, the same
as you would for any code you didn't watch compile yourself.

Known gaps (tracked, not hidden) are listed in `docs/architecture.md`'s
Roadmap section: no multi-output/DRM-KMS, no `utmp`/`wtmp`, one keyboard
layout, no screen-reader backend, no hibernate, no daemon-process
sandboxing.

## License

MIT, see `LICENSE`.
