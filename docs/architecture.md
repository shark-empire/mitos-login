# mitos-login architecture

## Process model

Three binaries/modes from one `mitos-login` executable, plus a separate
`mitos-loginctl`:

- **`mitos-login daemon`** -- root, one per machine. Owns the virtual
  terminal, the framebuffer, the input devices, authentication, lockout, the
  audit log, and session launching. Lives for the whole machine uptime
  (`services/mitos-login.service` restarts it if it ever exits).
- **`mitos-login greeter`** -- spawned by the daemon as an unprivileged
  system account (`greeter_user` in `config/login.toml`, default
  `mitos-greeter`), with `no_new_privs`, a fresh session, and exactly the fds
  the daemon decided to hand it: the framebuffer (fd 3), one end of a fresh
  `socketpair(2)` (fd 4), and the grabbed input devices (fd 5+). It never
  opens a device node or a config file itself -- see `src/daemon/lifecycle.rs`.
- **`mitos-loginctl`** -- an admin CLI that connects to the daemon's control
  socket (`/run/mitos-login/control.sock`, root-only).

This mirrors the traditional pre-`logind` display-manager split (the
greeter is a dumb, replaceable render/input process; the daemon holds every
credential and every privileged resource) rather than depending on
`libseat`/`logind`: since the daemon already runs as root, the permission
brokering `libseat` exists to provide for an *unprivileged* compositor is
not needed here.

## Why `/dev/fb0`, not DRM/KMS directly

`CONFIG_DRM_FBDEV_EMULATION` is on by default in virtually every distro
kernel, so `/dev/fb0` already exists, already mode-set to the boot
resolution, on top of whatever real KMS driver loaded -- no mode
enumeration or CRTC setup needed. This is the right tradeoff for a first,
fully-correct implementation: multi-output layouts and direct KMS access
are real gaps (see Roadmap below), not silently papered over.

## How this relates to `mitos-session`

`mitos-login` and `mitos-session` both do "authentication", at different
moments:

- `mitos-login` authenticates **before any session exists** -- the initial
  greeter, analogous to GDM/SDDM/LightDM.
- `mitos-session` re-authenticates **inside an already-running session** --
  the lock screen, rendered by `mitos-gui`, analogous to how `gnome-shell`
  owns its own screen lock via `logind` rather than going back through GDM.

`mitos-login`'s job ends the moment it execs the session's first process.
It discovers what to launch via `*.session` files under `sessions_dir`
(`config/session.toml`), the same convention X/Wayland display managers use
for `/usr/share/{x,wayland}sessions` -- it does not hard-code `mitos-session`
or any other sibling project's binary name. A `mitos-session` install is
expected to register itself with a `.session` file
(`Name=MITOS Desktop`, `Exec=/usr/bin/mitos-session`); from there, whatever
`mitos-session`'s own `launcher/` module does (spawning `mitos-gui`,
autostart apps) is that project's concern, not this one's.

## Configuration

Seven optional files under `/etc/mitos-login` (a missing file uses the
built-in default -- see `src/config/defaults.rs`):

| File              | Section      | Keys |
|-------------------|--------------|------|
| `login.toml`      | (top-level)  | `language`, `pam_library`, `greeter_user`, `sessions_dir`, `state_dir`, `control_socket_path`, `runtime_dir_base` |
| `greeter.toml`     | `[accessibility]` | `high_contrast`, `large_text`, `reduced_motion`, `screen_reader`, `visible_focus`; top-level `background_path` |
| `users.toml`      | `[list]`     | `min_uid`, `max_uid`, `denied_shells`, `always_hide`, `allow_only` |
| `security.toml`   | `[policy]`   | `free_attempts`, `lockout_base_secs`, `lockout_max_secs`, `auth_timeout_secs`, `greeter_idle_timeout_secs`, `max_secret_len`, `max_username_len`, `allow_password`, `allow_pin`, `restrict_root_to_console` |
| `session.toml`    | (top-level)  | `root_session`, `sessions_dir` |
| `display.toml`    | (top-level)  | `framebuffer_device`, `input_device_dir`, `greeter_vt` |
| `autologin.toml`  | (top-level)  | `user`, `delay_secs` |

`mitos-login check-config` validates a config directory and exits without
starting the daemon.

## Roadmap / known gaps

Tracked honestly rather than silently skipped:

- **Multi-output / DRM-KMS**: not implemented. `/dev/fb0` only, single
  output, native boot resolution.
- **`utmp`/`wtmp`**: not implemented, so `who`/`w` will not see mitos-login
  sessions. Needs its own verified struct-layout work (see
  `scripts/check-ffi.sh`'s pattern) not yet done.
- **Additional keyboard layouts**: only US QWERTY exists
  (`src/seats/input.rs`); the keymap table is the seam a second layout
  would extend.
- **Screen reader integration**: `AccessibilitySettings::screen_reader`
  exists as a flag with no backend behind it yet (needs a real
  `speech-dispatcher`-style integration to build against).
- **Hibernate (suspend-to-disk)**: deliberately refused
  (`src/power/hibernate.rs`) rather than guessed, since a wrong resume-offset
  guess risks data loss.
- **Daemon sandboxing**: the daemon runs as plain root for its whole
  lifetime (see `docs/security.md`); systemd capability/seccomp confinement
  is a real gap, not attempted as a partial measure.
