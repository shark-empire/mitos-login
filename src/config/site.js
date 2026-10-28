import { config } from './env';

export const site = {
  name: 'SCI-FI DATA',
  company: 'Sci-Fi Tech Inc',
  tagline: 'Data bundles and results checkers for Ghana',
  whatsappUrl: (text) =>
    `https://wa.me/${config.contact.whatsapp}${text ? `?text=${encodeURIComponent(text)}` : ''}`,
};

// The three networks. `id` is the canonical name used across the UI.
export const NETWORKS = [
  { id: 'MTN', label: 'MTN', long: 'MTN Ghana', lookupCode: 'MTN', meta: 'mtn', delivery: 'about 1 minute' },
  { id: 'Telecel', label: 'Telecel', long: 'Telecel (Vodafone)', lookupCode: 'VOD', meta: 'telecel', delivery: 'about 2 minutes' },
  { id: 'AirtelTigo', label: 'AirtelTigo', long: 'AirtelTigo', lookupCode: 'ATL', meta: 'airteltigo', delivery: 'about 2 minutes' },
];

export const networkById = (id) => NETWORKS.find((n) => n.id === id);

/** Map whatever the database stores ("Vodafone", "AT", "mtn") onto a canonical network id. */
export const normalizeNetwork = (value) => {
  const s = String(value ?? '').trim().toLowerCase();
  if (s.includes('mtn')) return 'MTN';
  if (/telecel|vodafone|^vod$/.test(s)) return 'Telecel';
  if (/airtel|tigo|^at$|^atl$/.test(s)) return 'AirtelTigo';
  return value ? String(value) : 'Other';
};

// Results checker products (from the Babor project).
export const RESULTS_REGULAR_PRICE = 20;
export const RESULTS_BULK_PRICE = 17;
export const RESULTS_BULK_THRESHOLD = 5; // buying MORE than this many gets the bulk price

export const RESULTS_PRODUCTS = [
  { id: 'wassce_checker', name: 'WASSCE', full: 'WASSCE results checker', desc: 'For school candidates sitting the WASSCE.' },
  { id: 'novdec_checker', name: 'NOVDEC', full: 'NOVDEC results checker', desc: 'For private candidates sitting the November/December WASSCE.' },
  { id: 'bece_checker', name: 'BECE', full: 'BECE results checker', desc: 'For Junior High School candidates sitting the BECE.' },
];

export const FAQ = [
  {
    q: 'What if I enter the wrong number?',
    a: 'Data sent to the wrong number cannot be recalled, so check it before you pay. After you enter a number we show the registered owner’s name so you can confirm it.',
  },
  {
    q: 'How long does delivery take?',
    a: 'Usually within 1 to 2 minutes. When networks are busy it can take up to 30 minutes. If it takes longer than that, message us on WhatsApp with your reference.',
  },
  {
    q: 'Is my payment secure?',
    a: 'Yes. Payments are processed by Paystack. We never see or store your card details or Mobile Money PIN.',
  },
  {
    q: 'Can I buy data for someone else?',
    a: 'Yes. Enter their number as the recipient. The bundle goes straight to that number, not yours.',
  },
  {
    q: 'How do I check an order?',
    a: 'Open Track order and enter the reference you got at checkout (it starts with SF-). You will see the delivery status and can ask for help from there.',
  },
  {
    q: 'Where do I find my results checker PIN?',
    a: 'Right after payment, on the results checker page. A copy is also sent to the email address you entered.',
  },
];
