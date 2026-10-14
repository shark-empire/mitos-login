# IPC

Two independent, hand-rolled wire protocols (zero dependencies -- a
deliberate choice for a crate that runs as root next to the login screen;
every dependency would be part of the trusted base). Framing is the same
for both: a 4-byte little-endian length prefix (capped at 1 MiB,
`protocol::MAX_FRAME_LEN`) followed by the payload.

## Greeter link

An anonymous `socketpair(2)` the daemon creates before spawning the
greeter and inherits into it across `exec` -- no path, no permission bits,
only that one process pair ever holds either end.

- **Greeter -> daemon** (`GreeterRequest`): `Ready`, `Authenticate { user,
  method, secret }`, `SelectSession { id }`, `PowerAction(Suspend | Reboot
  | PowerOff)`, `Heartbeat`.
- **Daemon -> greeter** (`GreeterEvent`): `Show { users, sessions,
  last_user }`, `AuthFeedback { outcome }`, `Shake`,
  `PowerCapabilities { .. }`, `PowerActionFailed { message }`, `Farewell`.

Every decoder ([`protocol::decode_greeter_request`] /
[`decode_greeter_event`]) is bounds-checked and returns an error rather
than panicking on truncated or malformed input -- the greeter is
unprivileged, so the daemon must never trust it, however unlikely a
malformed message is from code this project itself wrote.

## Control socket

A real, listening Unix socket (`/run/mitos-login/control.sock`, `0600`)
`mitos-loginctl` connects to. One connection, one request, one response,
then it closes -- matching `mitos-services`' own control-socket convention.

Authorization is `SO_PEERCRED`, not a password: the kernel-verified uid of
the connecting process must be `0`, checked in
[`ipc::permissions::check`](../src/ipc/permissions.rs) before the request
is even read. This is the real boundary; the socket's `0600` mode is
defense in depth on top of it, not instead of it.

Requests: `Status`, `ListSessions`, `Reload`, `VerifyAuditLog`,
`Terminate { session_id }`. See `mitos-loginctl help`.

## Message vocabulary

Field and enum names deliberately parallel `mitos-session`'s own IPC
(`AuthOutcome`, `Request`/`Event`) so the two daemons read as one
consistent design, even though they share no code and no wire format --
this one for the initial greeter, that one for the in-session lock screen.
