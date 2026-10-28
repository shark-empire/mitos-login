/**
 * The customer-facing store addresses: /s/kwame-data-hub and /s/kwame-data-hub/track.
 * These render without the site's menu or footer. `legacy` also accepts /store/<slug>, the older share links.
 */
export function matchShop(path, { legacy = false } = {}) {
  const m = /^\/s\/([^/]+)(\/track)?$/.exec(path) || (legacy ? /^\/store\/([^/]+)(\/track)?$/.exec(path) : null);
  if (!m) return null;
  try {
    return { slug: decodeURIComponent(m[1]), view: m[2] ? 'track' : 'shop' };
  } catch {
    return null;
  }
}
