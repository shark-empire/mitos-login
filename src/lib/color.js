export const DEFAULT_ACCENT = '#38BDF8';

/** Accept only #rrggbb. Store colours come from a database, so they never reach CSS unchecked. */
export const safeHex = (value, fallback = DEFAULT_ACCENT) => (/^#[0-9a-f]{6}$/i.test(String(value ?? '')) ? value : fallback);

/** Black or white text, whichever is readable on this colour. */
export const inkOn = (hex) => {
  const h = safeHex(hex).slice(1);
  const [r, g, b] = [0, 2, 4].map((i) => parseInt(h.slice(i, i + 2), 16));
  return (r * 299 + g * 587 + b * 114) / 1000 > 150 ? '#06120f' : '#ffffff';
};

export const ACCENTS = [
  { name: 'Sky', value: '#38BDF8' },
  { name: 'Mint', value: '#22D3A1' },
  { name: 'Amber', value: '#F59E0B' },
  { name: 'Red', value: '#EF4444' },
  { name: 'Violet', value: '#A78BFA' },
  { name: 'Pink', value: '#F472B6' },
];
