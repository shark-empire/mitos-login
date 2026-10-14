# Authentication

## Backends

[`detect_backend`](../src/authentication/client.rs) tries, in order:

1. **PAM**, via a `dlopen`'d `libpam.so.0` (never a build-time link --
   see `src/sys/pam.rs`). The service names used are `mitos-login`
   (password), `mitos-login-pin` (PIN), and `mitos-login-autologin`
   (account checks only, no credential prompt). Ship matching files under
   `/etc/pam.d/`.
2. **`crypt(3)` directly against `/etc/shadow`**, if libpam cannot be
   loaded at all (`Backend::ShadowOnly`). Password only -- PIN sign-in
   requires PAM.
3. **`Backend::Unavailable`** if neither loads. Every authentication
   attempt then returns `AuthOutcome::Error`, never a silent success.

## One request, one outcome

[`Authenticator::authenticate`](../src/authentication/client.rs) is the
single entry point:

1. Rejects implausible input (empty/oversized username or secret) before
   touching any backend.
2. Checks [`Lockout`](../src/security/lockout.rs); a locked-out account
   returns immediately without ever reaching PAM/`crypt`.
3. Runs the actual check, inside the policy's `auth_timeout_secs` budget.
4. Records the outcome (lockout counter + audit log) and returns
   [`AuthOutcome`](../src/authentication/response.rs): `Success`,
   `Failure`, `LockedOut { retry_after_secs }`, or `Error` (a backend
   problem, distinct from a wrong credential).

On `Success` with the PAM backend, the second return value is the *live*
`PamSession` -- already past `pam_authenticate`/`pam_acct_mgmt`/
`pam_setcred`, deliberately not dropped by `authenticate` itself. The
caller passes it to [`sessions::launcher`](../src/sessions/launcher.rs),
which calls `pam_open_session` and, later, `pam_close_session`, before the
session's own `pam_end` finally runs. Dropping it early would tear down
credentials a PAM module just established (resource limits, supplementary
groups) before the session ever starts.

## Account status without the hash

[`users::account::status_for`](../src/users/account.rs) reads
`/etc/shadow` for *display* purposes only (locked? expired?) -- it is safe
to compute for the list the unprivileged greeter renders, because it never
touches the hash field itself.
