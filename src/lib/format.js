export const money = (n) => `GH₵ ${Number(n || 0).toFixed(2)}`;

/** Amounts in the payments table are stored in pesewas. */
export const moneyFromPesewas = (n) => (n == null || n === '' ? 'N/A' : money(Number(n) / 100));

export const dateTime = (iso) => {
  if (!iso) return '';
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return '';
  return d.toLocaleString('en-GH', { dateStyle: 'medium', timeStyle: 'short' });
};

/** "5GB" -> { num: "5", unit: "GB" }. Anything unexpected is returned whole. */
export const splitSize = (size) => {
  const m = String(size ?? '').trim().match(/^(\d+(?:\.\d+)?)\s*([A-Za-z]+)$/);
  return m ? { num: m[1], unit: m[2].toUpperCase() } : { num: String(size ?? ''), unit: '' };
};

/** Sort key so 500MB < 1GB < 10GB. */
export const sizeInMb = (size) => {
  const { num, unit } = splitSize(size);
  const n = parseFloat(num);
  if (Number.isNaN(n)) return Number.MAX_SAFE_INTEGER;
  return unit === 'GB' ? n * 1024 : unit === 'TB' ? n * 1024 * 1024 : n;
};

const STATUS = {
  completed: { tone: 'ok', label: 'Delivered' },
  delivered: { tone: 'ok', label: 'Delivered' },
  success: { tone: 'ok', label: 'Delivered' },
  paid: { tone: 'ok', label: 'Paid' },
  processing: { tone: 'warn', label: 'Processing' },
  pending: { tone: 'warn', label: 'Processing' },
  failed: { tone: 'bad', label: 'Failed' },
};

export const statusInfo = (status) =>
  STATUS[String(status || 'processing').toLowerCase()] || { tone: 'muted', label: String(status) };
