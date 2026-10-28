export const digitsOnly = (v) => String(v ?? '').replace(/\D/g, '');

/**
 * Tidy a number as it is typed or pasted: digits only, "+233 54 123 4567" becomes "0541234567", at most 10 digits.
 * (A plain maxLength would cut a pasted "054 123 4567" short before the spaces were removed.)
 */
export const cleanPhone = (v) => {
  let d = digitsOnly(v);
  if (d.startsWith('233') && d.length >= 12) d = `0${d.slice(3)}`;
  return d.slice(0, 10);
};

// Ghana mobile prefixes (after the leading 0).
const PREFIXES = [20, 23, 24, 25, 26, 27, 28, 50, 53, 54, 55, 56, 57, 59];

/**
 * Validate a Ghana mobile number.
 * `strict` also checks the network prefix (used when creating an account).
 * Purchases use the loose check so a newly issued prefix never blocks a sale.
 */
export function validatePhone(value, { strict = false } = {}) {
  const d = digitsOnly(value);
  if (d.length !== 10) return { ok: false, msg: 'Enter all 10 digits' };
  if (!d.startsWith('0')) return { ok: false, msg: 'The number must start with 0' };
  if (strict && !PREFIXES.includes(parseInt(d.slice(1, 3), 10))) {
    return { ok: false, msg: 'That is not a Ghana network prefix' };
  }
  return { ok: true, msg: '' };
}

/** Accounts created with a phone number get a synthetic sign-in email. */
export const phoneToEmail = (phone) => `${digitsOnly(phone)}@sci-fidata.com`;
