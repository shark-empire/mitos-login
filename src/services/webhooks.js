// Calls to the Activepieces / Make automation endpoints.
import { config, isPlaceholder } from '../config/env';
import { NETWORKS } from '../config/site';

async function postJson(url, body, { signal } = {}) {
  let res;
  try {
    res = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
      signal,
    });
  } catch (err) {
    if (err.name === 'AbortError') throw err;
    throw new Error('Could not reach the service. Check your connection and try again.');
  }
  let data = null;
  try {
    data = await res.json();
  } catch {
    /* non-JSON response */
  }
  if (res.status === 429) throw new Error('Too many requests. Wait a few seconds and try again.');
  if (!res.ok) throw new Error(data?.error || data?.message || `The service returned an error (${res.status}).`);
  return data ?? {};
}

export const walletPaymentsEnabled = !isPlaceholder(config.hooks.walletPurchase);
export const resultsDeliveryConfigured = !isPlaceholder(config.hooks.resultsVerify);
export const dataTransferEnabled = !isPlaceholder(config.hooks.dataTransfer);

/** Who is the registered owner of this number on this network? */
export async function lookupOwner({ phone, network, signal }) {
  const lookupCode = NETWORKS.find((n) => n.id === network)?.lookupCode;
  return postJson(config.hooks.ownerLookup, { phoneNumber: String(phone), networkCode: lookupCode }, { signal });
}

/** Find orders by payment reference. */
export async function findOrdersByReference(reference) {
  return postJson(config.hooks.orderLookup, { type: 'reference', reference });
}

export async function purchaseWithWallet({ token, userId, bundleId, recipientPhone }) {
  return postJson(config.hooks.walletPurchase, {
    user_jwt: token,
    user_id: userId,
    bundle_id: bundleId,
    recipient_phone: recipientPhone,
  });
}

/** Exchange a paid Paystack reference for results checker PINs. */
export async function fetchResultsPins({ reference, productId, quantity }) {
  return postJson(config.hooks.resultsVerify, { reference, product_id: productId, quantity });
}

export async function sendDataTransfer(payload) {
  return postJson(config.hooks.dataTransfer, payload);
}
