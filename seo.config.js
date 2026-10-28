// Search-engine and link-preview tags for the main site, built from src/config/profile.json.
// Used by vite.config.js at build time. Social apps and search crawlers read the HTML as served,
// so these tags are written into index.html rather than set by the running app.
import fs from 'node:fs';

export function loadProfile() {
  const file = new URL('./src/config/profile.json', import.meta.url);
  try {
    return JSON.parse(fs.readFileSync(file, 'utf8'));
  } catch {
    // Some tools run this from a copy in a temporary folder; the project root is the working directory.
    return JSON.parse(fs.readFileSync(`${process.cwd()}/src/config/profile.json`, 'utf8'));
  }
}

const esc = (s) => String(s ?? '').replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
const text = (v) => String(v ?? '').trim();
const root = (p) => text(p.siteUrl).replace(/\/+$/, '');

/** Drop empty strings, empty arrays and empty objects so unfilled details are simply left out. */
function prune(v) {
  if (Array.isArray(v)) {
    const a = v.map(prune).filter((x) => x !== undefined);
    return a.length ? a : undefined;
  }
  if (v && typeof v === 'object') {
    const o = Object.fromEntries(Object.entries(v).map(([k, x]) => [k, prune(x)]).filter(([, x]) => x !== undefined));
    return Object.keys(o).length ? o : undefined;
  }
  return typeof v === 'string' ? (v.trim() ? v : undefined) : v;
}

/** schema.org description of the organization, the website and (when named) the founder. */
export function structuredData(p) {
  const url = root(p);
  const org = p.organization || {};
  const founder = p.founder || {};
  const orgId = `${url}/#organization`;
  const founderId = `${url}/#founder`;
  const graph = [
    {
      '@type': 'Organization',
      '@id': orgId,
      name: p.brand,
      legalName: text(org.legalName),
      alternateName: (p.alternateNames || []).filter((n) => n !== p.brand),
      url: `${url}/`,
      logo: { '@type': 'ImageObject', url: `${url}/logo.svg` },
      image: `${url}/og-image.jpg`,
      description: p.description,
      foundingDate: text(org.foundingYear),
      founder: text(founder.name) ? { '@id': founderId } : undefined,
      areaServed: text(org.country) ? { '@type': 'Country', name: org.country } : undefined,
      address: text(org.countryCode) ? { '@type': 'PostalAddress', addressCountry: org.countryCode } : undefined,
      contactPoint: text(p.phone) || text(p.email)
        ? [{ '@type': 'ContactPoint', contactType: 'customer support', telephone: text(p.phone), email: text(p.email), availableLanguage: 'English', areaServed: text(org.countryCode) }]
        : undefined,
      sameAs: (p.social || []).map(text).filter(Boolean),
    },
    {
      '@type': 'WebSite',
      '@id': `${url}/#website`,
      url: `${url}/`,
      name: p.brand,
      alternateName: (p.alternateNames || []).filter((n) => n !== p.brand),
      inLanguage: 'en-GH',
      publisher: { '@id': orgId },
    },
  ];
  if (text(founder.name)) {
    graph.push({
      '@type': 'Person',
      '@id': founderId,
      name: text(founder.name),
      jobTitle: text(founder.title) || 'Founder',
      description: text(founder.about),
      worksFor: { '@id': orgId },
      sameAs: [],
    });
  }
  return prune({ '@context': 'https://schema.org', '@graph': graph });
}

/** The words shown when JavaScript is off; crawlers and some link scrapers read them too. */
export function plainSummary(p) {
  const org = p.organization || {};
  const founder = p.founder || {};
  const names = (p.alternateNames || []).filter((n) => n !== p.brand);
  return [
    `${p.brand}${names.length ? ` (also written ${names.slice(0, 3).join(', ')})` : ''} sells MTN, Telecel and AirtelTigo data bundles and WAEC results checkers in ${text(org.country) || 'Ghana'}.`,
    `Website: ${text(p.siteUrl).replace(/^https?:\/\//, '').replace(/\/+$/, '')}.`,
    text(p.phone) ? `WhatsApp: ${p.phone}.` : '',
    text(org.legalName) ? `${p.brand} is operated by ${org.legalName}.` : '',
    text(founder.name) ? `Founder: ${founder.name}.` : '',
    'Please turn on JavaScript to place an order.',
  ].filter(Boolean).join(' ');
}

/** The tags for <head>, as one string. */
export function seoHead(p) {
  const url = root(p);
  const image = `${url}/og-image.jpg`;
  const alt = `${p.brand}: affordable data bundles`;
  const v = p.verification || {};
  const lines = [
    `<link rel="canonical" href="${esc(url)}/" />`,
    '<meta name="robots" content="index,follow,max-image-preview:large" />',
    `<meta name="application-name" content="${esc(p.brand)}" />`,
    `<meta name="apple-mobile-web-app-title" content="${esc(p.brand)}" />`,
    '<meta property="og:type" content="website" />',
    `<meta property="og:site_name" content="${esc(p.brand)}" />`,
    `<meta property="og:title" content="${esc(p.title)}" />`,
    `<meta property="og:description" content="${esc(p.description)}" />`,
    `<meta property="og:url" content="${esc(url)}/" />`,
    `<meta property="og:image" content="${esc(image)}" />`,
    '<meta property="og:image:type" content="image/jpeg" />',
    '<meta property="og:image:width" content="1200" />',
    '<meta property="og:image:height" content="630" />',
    `<meta property="og:image:alt" content="${esc(alt)}" />`,
    '<meta property="og:locale" content="en_GH" />',
    '<meta name="twitter:card" content="summary_large_image" />',
    `<meta name="twitter:title" content="${esc(p.title)}" />`,
    `<meta name="twitter:description" content="${esc(p.description)}" />`,
    `<meta name="twitter:image" content="${esc(image)}" />`,
    `<meta name="twitter:image:alt" content="${esc(alt)}" />`,
    text(v.google) ? `<meta name="google-site-verification" content="${esc(text(v.google))}" />` : '',
    text(v.bing) ? `<meta name="msvalidate.01" content="${esc(text(v.bing))}" />` : '',
    // "<" is escaped so nothing in the profile can close the script tag early.
    `<script type="application/ld+json">${JSON.stringify(structuredData(p)).replace(/</g, '\\u003c')}</script>`,
  ].filter(Boolean);
  return lines.map((l) => `    ${l}`).join('\n');
}

/** Turn the plain index.html into the search-ready one. */
export function applySeo(html, p) {
  let out = html
    .replace(/<title>[\s\S]*?<\/title>/, `<title>${esc(p.title)}</title>`)
    .replace(/<meta\s+name="description"[^>]*>/, `<meta name="description" content="${esc(p.description)}" />`)
    .replace(/<noscript>[\s\S]*?<\/noscript>/, `<noscript>${esc(plainSummary(p))}</noscript>`);
  if (!out.includes('<!-- seo:head -->')) throw new Error('index.html is missing the <!-- seo:head --> marker');
  out = out.replace('<!-- seo:head -->', seoHead(p).trimStart());
  return out;
}

export const robotsTxt = (p) => `User-agent: *\nAllow: /\n\nSitemap: ${root(p)}/sitemap.xml\n`;

export const sitemapXml = (p, date = new Date()) =>
  `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n  <url>\n    <loc>${esc(root(p))}/</loc>\n    <lastmod>${date.toISOString().slice(0, 10)}</lastmod>\n    <changefreq>weekly</changefreq>\n    <priority>1.0</priority>\n  </url>\n</urlset>\n`;
