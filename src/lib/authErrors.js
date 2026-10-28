// Turn a sign-in / sign-up failure into a message a customer can act on.
// GoTrue (Supabase auth) sends a machine code and an English message; we match on both because
// older projects only send the message.

const has = (err, ...needles) => {
  const hay = `${err?.code || ''} ${err?.message || ''}`.toLowerCase();
  return needles.some((n) => hay.includes(n));
};

/** True when the server refused a login because the sign-up email was never confirmed. */
export const isUnconfirmed = (err) => has(err, 'email_not_confirmed', 'email not confirmed');

/** Reasons that have nothing to do with what the person typed. */
function commonProblem(err) {
  if (!err) return null;
  if (err.code === 'network' || err.name === 'TypeError') {
    return 'We can’t reach the server. Check your internet connection and try again.';
  }
  if (err.status === 429 || has(err, 'rate limit', 'over_request_rate', 'too many')) {
    return 'Too many attempts. Wait a few minutes and try again.';
  }
  if (has(err, 'api key', 'apikey')) {
    return 'The site’s connection key was rejected. Please contact support.';
  }
  if (err.status >= 500) return 'The server had a problem. Try again in a moment.';
  return null;
}

/**
 * @param {Error & {status?: number, code?: string}} err
 * @param {'email'|'phone'} kind  how the person identified themselves
 * @param {string} [email]        the address, so we can name it in the message
 */
export function loginError(err, kind = 'email', email = '') {
  const common = commonProblem(err);
  if (common) return common;

  if (isUnconfirmed(err)) {
    return kind === 'phone'
      ? 'This account has not been activated yet. Contact support and we will activate it.'
      : `Your email isn’t confirmed yet. Open the confirmation link we sent${email ? ` to ${email}` : ''}, then log in.`;
  }
  if (has(err, 'invalid_credentials', 'invalid login', 'invalid_grant', 'invalid credentials')) {
    return kind === 'phone' ? 'Wrong phone number or password.' : 'Wrong email or password.';
  }
  if (has(err, 'user_banned', 'banned')) return 'This account has been suspended. Contact support.';
  return err?.message ? `We couldn’t log you in: ${err.message}` : 'We couldn’t log you in. Try again.';
}

export function signupError(err, kind = 'email') {
  if (has(err, 'user_already_exists', 'already registered', 'already been registered', 'already exists')) {
    return kind === 'phone'
      ? 'This phone number already has an account. Log in instead.'
      : 'This email already has an account. Log in instead.';
  }
  if (has(err, 'over_email_send_rate_limit', 'email rate limit')) {
    return 'Too many sign-up emails have been sent for now. Wait about an hour, or sign up with your phone number instead.';
  }
  const common = commonProblem(err);
  if (common) return common;

  if (has(err, 'email_address_invalid', 'email address') && has(err, 'invalid')) {
    return 'That email address was not accepted. Check the spelling, or try a different address.';
  }
  if (has(err, 'email_address_not_authorized', 'not authorized')) {
    return 'Emails can’t be sent to that address yet. Try another address or sign up with your phone number.';
  }
  if (has(err, 'signup_disabled', 'signups not allowed', 'signup is disabled')) {
    return 'New sign-ups are switched off right now. Contact support.';
  }
  if (has(err, 'weak_password', 'password should')) {
    return 'Choose a stronger password (at least 8 characters, with a capital letter and a number).';
  }
  return err?.message ? `We couldn’t create your account: ${err.message}` : 'We couldn’t create your account. Try again.';
}
