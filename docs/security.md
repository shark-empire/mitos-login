# Security design

## Threat model

The daemon is root and faces input from two directions: the network has no
role here at all (this is a local console login manager), so the two
untrusted-ish boundaries are (1) whatever a person standing at the keyboard
types, and (2) the greeter process itself, which is unprivileged but still
a separate process whose messages the daemon must not blindly trust.

## Process separation

The greeter runs as a dedicated system account (`greeter_user`, see
`config/login.toml`) with `no_new_privs` set and a session of its own. It
never touches `/etc/shadow`, PAM, the lockout state, or the audit log --
only the daemon does. A compromised greeter can misbehave over its one
IPC link (see `src/ipc/protocol.rs`'s bounds-checked reader) but cannot
itself authenticate, read another account's credentials, or write to the
tamper-evident log.

## Credential handling

- Typed passwords/PINs are held in [`Secret`](../src/security/secret.rs):
  `mlock`'d on a best-effort basis, overwritten with zeros (via
  `ptr::write_volatile`, not a plain assignment the optimizer could elide)
  the moment they are consumed or dropped.
- Comparison against anything secret-shaped uses
  [`ct_eq`](../src/security/secret.rs) (constant-time), never `==`.
- PAM is tried first; `crypt(3)` directly against `/etc/shadow` is the
  fallback for systems with no usable PAM stack (see
  `src/authentication/shadow.rs`). Neither path is a hand-rolled hash --
  this crate never implements its own password hashing.
- `crypt(3)` is loaded via `dlopen` from a root-owned, non-group/other-
  writable library file only (`src/sys/system_library_candidates`),
  mirroring the same check `config` files and `.session` files get
  (`src/security/permissions.rs`).

## Lockout

Exponential backoff per account (`security.toml`'s `free_attempts`/
`lockout_base_secs`/`lockout_max_secs`), persisted to
`/var/lib/mitos-login/lockout.state` so a daemon restart cannot reset an
attacker's counter. The file is HMAC-SHA256-signed with a machine-local key
(`lockout.key`, `0600`, generated once from `/dev/urandom`) -- this is a
speed bump against a local filesystem bug or race, not the primary control
(the real boundary is the file's `0600` root-only permissions). A failed
signature check discards the state (fails toward "nothing learned yet"
rather than trusting a possibly-forged counter) and is itself logged as an
`integrity_violation` audit event.

## Audit log

`/var/lib/mitos-login/audit.jsonl`: one JSON object per line, each carrying
a SHA-256 hash over its own fields plus the previous line's hash (see
`src/security/audit.rs`). Anyone with read access can verify the chain
(`mitos-loginctl audit-verify` does exactly this); tampering with any line
but the last breaks every hash after it. This is tamper-*evident*, not
tamper-*proof* -- an attacker with root can still rewrite the whole file
and re-chain it from scratch. It raises the bar from "silently edit one
line" to "rewrite the entire log convincingly," and a backup/syslog
forward of this file closes that last gap (not implemented here).

## Privilege drop

Every spawned process (`src/sys::spawn`) drops privileges in the POSIX-
mandated order -- `setgroups` -> `setgid` -> `setuid` -- then immediately
verifies `setuid(0)`/`setgid(0)` both fail, so a bug that left the process
still capable of regaining root is caught at spawn time rather than
discovered later. This exact sequence was exercised against a real
unprivileged account during development (fork, drop to that account,
confirm `setuid(0)` returns `EPERM`), not merely reviewed.

## Root login

`root` may only authenticate from a physical console seat (`ttyN`), never
a remote or virtual one (`security.toml`'s `restrict_root_to_console`,
default on). A successful root login never gets the graphical desktop
session -- see `config/session.toml`'s `root_session`.

## What this project does not attempt

- **Its own crypto primitives for passwords.** SHA-256/HMAC-SHA256 exist in
  this crate (`src/security/hash.rs`) only for the audit-log hash chain and
  the lockout-state signature -- never for a password itself, which always
  goes through the system's real `crypt(3)`/PAM.
- **Sandboxing the daemon process itself** (seccomp, capability dropping).
  It remains plain root for its whole lifetime. A correct sandbox would
  need to enumerate everything the greeter/session spawn path requires
  (VT and device ioctls, `setuid`/`setgid`, `reboot(2)`) and is tracked as
  a gap, not attempted partially.
- **Hibernate.** See `docs/architecture.md`'s roadmap -- refusing is safer
  than guessing a resume offset.
