// Browser storage can throw (private windows, blocked cookies), so every access is guarded.
const make = (getStore) => ({
  get(key, fallback = null) {
    try {
      const v = getStore().getItem(key);
      return v == null ? fallback : v;
    } catch {
      return fallback;
    }
  },
  set(key, value) {
    try {
      getStore().setItem(key, value);
    } catch {
      /* ignore */
    }
  },
  remove(key) {
    try {
      getStore().removeItem(key);
    } catch {
      /* ignore */
    }
  },
  getJson(key, fallback) {
    try {
      const v = getStore().getItem(key);
      return v == null ? fallback : JSON.parse(v);
    } catch {
      return fallback;
    }
  },
  setJson(key, value) {
    this.set(key, JSON.stringify(value));
  },
});

export const local = make(() => window.localStorage);
export const session = make(() => window.sessionStorage);

const RECENT_KEY = 'sf_recent_refs';
export const recentRefs = {
  list: () => local.getJson(RECENT_KEY, []),
  add(ref) {
    if (!ref) return;
    const next = [ref, ...this.list().filter((r) => r !== ref)].slice(0, 8);
    local.setJson(RECENT_KEY, next);
  },
};

// Orders placed from a store on this device, so a customer can find them again on that store's tracking page.
const SHOP_KEY = 'sf_shop_orders';
export const shopOrders = {
  list(slug) {
    const all = local.getJson(SHOP_KEY, {});
    return Array.isArray(all?.[slug]) ? all[slug] : [];
  },
  add(slug, order) {
    if (!slug || !order?.ref) return;
    const all = local.getJson(SHOP_KEY, {}) || {};
    const rest = (Array.isArray(all[slug]) ? all[slug] : []).filter((o) => o.ref !== order.ref);
    all[slug] = [order, ...rest].slice(0, 8);
    local.setJson(SHOP_KEY, all);
  },
};
