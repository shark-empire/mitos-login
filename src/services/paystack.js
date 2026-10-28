// Paystack inline checkout, loaded on demand so the script only downloads when someone is about to pay.
let loading = null;

export function loadPaystack() {
  if (typeof window !== 'undefined' && window.PaystackPop) return Promise.resolve();
  if (!loading) {
    loading = new Promise((resolve, reject) => {
      const s = document.createElement('script');
      s.src = 'https://js.paystack.co/v1/inline.js';
      s.async = true;
      s.onload = () => resolve();
      s.onerror = () => {
        loading = null;
        s.remove();
        reject(new Error('Could not load Paystack. Check your connection and try again.'));
      };
      document.head.appendChild(s);
    });
  }
  return loading;
}

/**
 * Open the Paystack popup.
 * `amount` is in cedis; Paystack wants pesewas.
 * Paystack requires plain (non-async) callbacks, so keep them synchronous.
 */
export async function openPaystack({ key, email, amount, reference, metadata, onSuccess, onClose }) {
  await loadPaystack();
  const handler = window.PaystackPop.setup({
    key,
    email,
    amount: Math.round(amount * 100),
    currency: 'GHS',
    ref: reference,
    metadata,
    callback: (response) => onSuccess(response),
    onClose: () => onClose && onClose(),
  });
  handler.openIframe();
}
