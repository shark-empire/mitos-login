# Troubleshooting

## The daemon won't start

Run `mitos-login check-config` first -- a malformed config file is a loud,
specific error (file + line number), not a silent fallback. If that
passes, check:

- `greeter_user` (default `mitos-greeter`) must exist:
  `useradd --system --no-create-home --shell /usr/sbin/nologin mitos-greeter`.
- `framebuffer_device` (default `/dev/fb0`) must exist and be readable.
  If it doesn't, your kernel likely has `CONFIG_DRM_FBDEV_EMULATION`
  disabled, or no KMS driver has loaded yet -- see
  `docs/architecture.md`'s note on why this project targets `/dev/fb0`.
- The daemon needs a free virtual terminal; if VT allocation fails, check
  nothing else (a getty, another display manager) already owns the one
  `greeter_vt` names, or leave `greeter_vt` unset to let the kernel pick.

## The greeter shows up but nothing is drawn / is garbled

Check `info.bpp` and the channel bit layout your framebuffer reports --
`Display::open` and `Canvas::blit_to_fb` both refuse (loudly) anything
outside 8-32 bpp or a channel width other than 5/6/8 bits, rather than
drawing a corrupted image. If you hit this refusal, your framebuffer mode
is real but unusual; file it rather than working around it blindly.

## Typing does nothing

The greeter grabs (`EVIOCGRAB`) every keyboard/pointer-capable device it
can open under `input_device_dir` (default `/dev/input`) at startup, so a
device plugged in *after* the greeter starts will not be seen until it
restarts. Only US QWERTY is implemented (`src/seats/input.rs`) -- a
different physical layout will produce the wrong characters for
punctuation/shifted keys even though letters/digits still line up.

## "Account locked" / "too many attempts" immediately after a restart

This is by design -- `security.toml`'s lockout state persists across
daemon restarts specifically so restarting the daemon cannot reset an
attacker's counter (see `docs/security.md`). Check
`/var/lib/mitos-login/lockout.state`'s age/content, or wait out the
backoff window.

## `mitos-loginctl` says "cannot reach the daemon" / "not permitted"

`mitos-loginctl` must run as root (the control socket is `SO_PEERCRED`-gated
to uid 0 -- see `docs/ipc.md`) and the daemon must actually be running
(`systemctl status mitos-login` or your supervisor's equivalent).

## `mitos-loginctl audit-verify` reports "TAMPERED"

Treat this as a real incident, not a glitch: it means the hash chain in
`/var/lib/mitos-login/audit.jsonl` is broken somewhere before the last
line written. See `docs/security.md`'s audit-log section for exactly what
this does and does not prove.

## I changed a config file and nothing happened

Most keys apply on daemon start or `mitos-loginctl reload`
(`ControlRequest::Reload` re-reads every config file and rebuilds the user
list/session discovery). `framebuffer_device`, `input_device_dir`,
`greeter_vt`, and `greeter_user` need a full daemon restart -- they are
read once at startup, before the seat is acquired.
