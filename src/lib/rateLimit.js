// Tiny client-side throttle to slow down accidental double submits and brute-force attempts.
// This is a courtesy, not security: real limits must live on the server.
const hits = {};

export function checkRate(key, max, windowMs) {
  const now = Date.now();
  hits[key] = (hits[key] || []).filter((t) => now - t < windowMs);
  if (hits[key].length >= max) {
    return { ok: false, waitSeconds: Math.ceil((windowMs - (now - hits[key][0])) / 1000) };
  }
  hits[key].push(now);
  return { ok: true, waitSeconds: 0 };
}
