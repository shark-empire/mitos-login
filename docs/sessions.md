# Sessions

## Discovery

[`sessions::selection::discover`](../src/sessions/selection.rs) reads every
trusted (root-owned, non-group/other-writable) `*.session` file directly
under `sessions_dir` (default `/usr/share/mitos-sessions`), the same
`Name=`/`Exec=` convention X/Wayland display managers use for
`/usr/share/{x,wayland}sessions`:

```
# /usr/share/mitos-sessions/mitos.session
Name=MITOS Desktop
Exec=/usr/bin/mitos-session
```

This project does not hard-code `mitos-session` or any other binary name --
see `docs/architecture.md` for how the two projects relate.

## Launch sequence

[`sessions::launcher::launch`](../src/sessions/launcher.rs), given a
successful `AuthOutcome::Success`:

1. Creates (or repairs the ownership/mode of)
   `XDG_RUNTIME_DIR` = `<runtime_dir_base>/<uid>`, `0700`, owned by the
   target user.
2. Calls `pam_open_session` on the live `PamSession` handed up from
   authentication (see `docs/authentication.md`), if there is one.
3. Builds the environment from scratch (`sessions::environment`) --
   `HOME`/`USER`/`LOGNAME`/`SHELL`/`PATH`/the `XDG_*` variables/`LANG` --
   then layers in whatever PAM modules asked to set via
   `pam_getenvlist`, **without** letting a PAM variable override identity
   fields a PAM module has no business changing.
4. Resolves the account's full supplementary group list
   (`getgrouplist(3)`) and spawns the session's `Exec=` command through
   [`sys::spawn`](../src/sys/mod.rs)'s privilege-dropping path.
5. On spawn failure, closes the PAM session again before returning the
   error -- never leaves a half-opened PAM session behind.

## Reaping

The daemon never blocks on a session: it tracks the launched pid in
[`SessionManager`](../src/sessions/manager.rs) and reaps it later via its
own `SIGCHLD`-driven, non-blocking `waitpid` loop
(`sys::wait_any_nonblocking`). Once reaped, `pam_close_session` runs and
only then does the `PamSession` drop (`pam_end`) -- bracketing the whole
session's lifetime, not just the login moment.

## Root

A successful `root` login (console-only, see `docs/security.md`) gets
`session.toml`'s `root_session` (default: the account's own login shell),
never a `*.session` entry -- root is an administrative account, not a
desktop user.

## Known gap

No `utmp`/`wtmp` writes yet, so `who`/`w` will not show mitos-login
sessions -- see `docs/architecture.md`'s roadmap.
