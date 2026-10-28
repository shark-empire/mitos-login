// Everything the reseller-store features read and write (the Store project's Supabase tables).
// Tables: stores, bundles, orders. Storage bucket: assets. Optional function: get_store_stats.
import { storesApi } from './rest';
import { normalizeNetwork } from '../config/site';
import { safeHex } from '../lib/color';

const enc = encodeURIComponent;

/** The Store project calls AirtelTigo "AT". The rest of this site calls it AirtelTigo. */
export const toDbNetwork = (id) => (id === 'AirtelTigo' ? 'AT' : id);

const sizeLabel = (gb) => (gb < 1 ? `${Math.round(gb * 1000)}MB` : `${Number(gb.toFixed(2))}GB`);

/** "90DAYS" -> "90 days", "24 Hours" -> "24 hours". */
export function validityLabel(v) {
  const s = String(v ?? '').trim();
  if (!s) return 'No expiry';
  const spaced = s.replace(/(\d)([a-z])/gi, '$1 $2').toLowerCase();
  return spaced.charAt(0).toUpperCase() + spaced.slice(1);
}

export const VALIDITIES = ['No expiry', '24 hours', '7 days', '30 days', '90 days'];

const toStore = (r) =>
  r && {
    id: r.id,
    name: r.name,
    slug: r.slug,
    description: r.description ?? '',
    logo: r.logo_url ?? null,
    accent: safeHex(r.accent_color),
    whatsapp: r.whatsapp ?? '',
    ownerId: r.owner_id,
    active: r.is_active !== false,
    createdAt: r.created_at,
  };

const toBundle = (r) => {
  const gb = Number(r.size_gb) || 0;
  return {
    id: r.id,
    name: r.name,
    size: sizeLabel(gb),
    sizeGb: gb,
    price: Number(r.price) || 0,
    network: normalizeNetwork(r.network),
    storeId: r.store_id,
    store: r.store ? toStore(r.store) : null,
    validity: validityLabel(r.validity),
    active: r.is_active !== false,
    createdAt: r.created_at,
  };
};

const toOrder = (r) => ({
  id: r.id,
  bundleId: r.bundle_id,
  storeId: r.store_id,
  phone: r.customer_phone,
  amount: Number(r.amount) || 0,
  status: r.status,
  reference: r.paystack_reference,
  method: r.payment_method || 'paystack',
  createdAt: r.created_at,
});

const WITH_STORE = 'select=*,store:stores(id,name,slug,logo_url,accent_color,whatsapp,is_active)';

/* ---------- public reads ---------- */

/** Everything on sale across all stores. */
export async function fetchBundles() {
  const rows = await storesApi.select('bundles', `?${WITH_STORE}&is_active=eq.true&order=created_at.desc&limit=300`);
  return (rows || []).map(toBundle).filter((b) => b.store && b.store.active);
}

export async function fetchBundleById(id) {
  const rows = await storesApi.select('bundles', `?${WITH_STORE}&id=eq.${enc(id)}&limit=1`);
  const b = rows?.[0] && toBundle(rows[0]);
  return b && b.active && b.store?.active ? b : null;
}

export async function fetchStores() {
  const rows = await storesApi.select('stores', '?select=*&is_active=eq.true&order=created_at.desc&limit=300');
  return (rows || []).map(toStore);
}

export async function fetchStoreBySlug(slug) {
  const rows = await storesApi.select('stores', `?select=*&slug=eq.${enc(slug)}&is_active=eq.true&limit=1`);
  return toStore(rows?.[0]) || null;
}

/** A store's bundles. A seller passes their token to see hidden ones as well. */
export async function fetchBundlesByStore(storeId, { token, includeHidden = false } = {}) {
  const hidden = includeHidden ? '' : '&is_active=eq.true';
  const rows = await storesApi.select('bundles', `?select=*&store_id=eq.${enc(storeId)}${hidden}&order=price.asc`, token);
  return (rows || []).map(toBundle);
}

/** Record a paid order. The row starts as `pending`: the payment automation verifies the reference and delivers. */
export function createOrder({ bundle, phone, reference }) {
  return storesApi.insert('orders', {
    bundle_id: bundle.id,
    store_id: bundle.storeId,
    customer_phone: phone,
    amount: bundle.price,
    paystack_reference: reference,
    payment_method: 'paystack',
    status: 'pending',
  });
}

/* ---------- seller ---------- */

export async function fetchStoreByOwner(ownerId, token) {
  const rows = await storesApi.select('stores', `?select=*&owner_id=eq.${enc(ownerId)}&limit=1`, token);
  return toStore(rows?.[0]) || null;
}

export async function isSlugTaken(slug) {
  const rows = await storesApi.select('stores', `?select=id&slug=eq.${enc(slug)}&limit=1`);
  return (rows || []).length > 0;
}

export function createStore({ ownerId, name, slug, description, whatsapp, accent, logoUrl }, token) {
  return storesApi.insert(
    'stores',
    {
      owner_id: ownerId,
      name,
      slug,
      description,
      whatsapp: whatsapp || null,
      accent_color: accent,
      logo_url: logoUrl || null,
    },
    token,
  );
}

export function uploadStoreLogo(file, ownerId, token) {
  const ext = (file.name.split('.').pop() || 'png').toLowerCase().replace(/[^a-z0-9]/g, '').slice(0, 5) || 'png';
  return storesApi.upload('assets', `store-logos/${ownerId}-${Date.now()}.${ext}`, file, token);
}

export async function fetchOrdersByStore(storeId, token, limit = 200) {
  const rows = await storesApi.select(
    'orders',
    `?select=*&store_id=eq.${enc(storeId)}&order=created_at.desc&limit=${limit}`,
    token,
  );
  return (rows || []).map(toOrder);
}

export function createBundle({ storeId, name, network, sizeGb, price, validity }, token) {
  return storesApi.insert(
    'bundles',
    {
      store_id: storeId,
      name,
      network: toDbNetwork(network),
      size_gb: Number(sizeGb),
      price: Number(price),
      validity,
      is_active: true,
    },
    token,
  );
}

export function setBundleActive(id, active, token) {
  return storesApi.update('bundles', `?id=eq.${enc(id)}`, { is_active: active }, token);
}

/** Totals from the database function, when it exists. Returns null otherwise so the page can work them out itself. */
export async function fetchStoreStats(storeId, token) {
  try {
    const data = await storesApi.rpc('get_store_stats', { p_store_id: storeId }, token);
    const row = Array.isArray(data) ? data[0] : data;
    if (!row) return null;
    return {
      sales: Number(row.total_sales ?? 0),
      today: Number(row.orders_today ?? 0),
      orders: Number(row.total_orders ?? 0),
      customers: Number(row.unique_customers ?? 0),
    };
  } catch {
    return null;
  }
}

/* ---------- order tracking (public, by payment reference) ---------- */

const maskPhone = (p) => {
  const d = String(p ?? '').replace(/\D/g, '');
  return d.length >= 7 ? `${d.slice(0, 3)}****${d.slice(-3)}` : '';
};

const toTracked = (r) => ({
  status: r.status,
  amount: Number(r.amount) || 0,
  createdAt: r.created_at,
  bundleName: r.bundle_name || '',
  size: Number(r.size_gb) ? sizeLabel(Number(r.size_gb)) : r.bundle_name || '',
  network: normalizeNetwork(r.network),
  phone: r.phone || '',
  storeName: r.store_name || '',
  storeSlug: r.store_slug || '',
  reference: r.reference || '',
});

/**
 * Look an order up by its payment reference. Returns the order, or null when there is none.
 * It calls the database function `track_store_order` (see the README for the SQL), which returns only
 * what a customer needs to see and a masked phone number. Without the function it falls back to reading the
 * orders table directly, which only works if your policies allow it.
 */
export async function trackOrder(reference) {
  const ref = String(reference ?? '').trim();
  if (!ref) return null;
  try {
    const row = await storesApi.rpc('track_store_order', { p_reference: ref });
    const one = Array.isArray(row) ? row[0] : row;
    return one && typeof one === 'object' ? toTracked({ ...one, reference: ref }) : null;
  } catch (err) {
    if (err.status !== 404 && err.code !== 'PGRST202') throw err;
  }
  const rows = await storesApi.select(
    'orders',
    `?paystack_reference=eq.${enc(ref)}&select=status,amount,created_at,customer_phone,bundle_id,store_id&limit=1`,
  );
  const o = rows?.[0];
  if (!o) return null;
  // The bundle's details are optional: a bundle the store has hidden since still tracks fine.
  const b = o.bundle_id
    ? (await storesApi.select('bundles', `?id=eq.${enc(o.bundle_id)}&select=name,size_gb,network,store:stores(name,slug)&limit=1`).catch(() => []))?.[0]
    : null;
  return toTracked({
    status: o.status,
    amount: o.amount,
    created_at: o.created_at,
    bundle_name: b?.name,
    size_gb: b?.size_gb,
    network: b?.network,
    phone: maskPhone(o.customer_phone),
    store_name: b?.store?.name,
    store_slug: b?.store?.slug,
    reference: ref,
  });
}
