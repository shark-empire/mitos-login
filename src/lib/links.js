import { config } from '../config/env';

/** A link a customer can open, built from wherever the site is hosted (works in a sub-folder and with hash routing). */
export const siteUrl = (path) => {
  const { origin, pathname } = window.location;
  return `${origin}${pathname}#${path}`;
};

/** The in-app path of a store's customer-facing page. It opens a view with no site menu, only the shop and order tracking. */
export const shopPath = (slug) => `/s/${encodeURIComponent(slug)}`;

/** The link a store owner shares with customers: on the store domain when one is configured, otherwise on this site. */
export const storeUrl = (slug) =>
  config.stores.linkBase ? `${config.stores.linkBase}/#${shopPath(slug)}` : siteUrl(shopPath(slug));

/** WhatsApp chat link from a number stored as digits. Returns '' when there is no usable number. */
export const whatsappLink = (digits, text) => {
  const n = String(digits ?? '').replace(/\D/g, '');
  if (n.length < 9) return '';
  return `https://wa.me/${n}${text ? `?text=${encodeURIComponent(text)}` : ''}`;
};
