import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';
import { fileURLToPath } from 'node:url';
import { applySeo, loadProfile, robotsTxt, sitemapXml } from './seo.config.js';

const here = (p) => fileURLToPath(new URL(p, import.meta.url));

// With VITE_SITE_MODE=stores the build contains only the store pages (a customer's storefront and order tracking).
// The main site's pages are never imported, so they are not in the bundle, and the HTML shell carries no brand.
const neutralHtml = () => ({
  name: 'stores-neutral-html',
  transformIndexHtml: (html) =>
    html
      .replace(/<title>[\s\S]*?<\/title>/, '<title>Shop</title>')
      .replace(/<meta\s+name="description"[^>]*>\s*/, '')
      .replace(/<link rel="icon"[^>]*>/, '<link rel="icon" href="data:," />\n    <meta name="robots" content="noindex,nofollow" />')
      .replace(/SCI-FI DATA needs JavaScript to run\. Please enable it and reload\./, 'Please turn on JavaScript to use this page.'),
});

// The full site gets search and link-preview tags, robots.txt and a sitemap, all from src/config/profile.json.
const seo = () => {
  const profile = loadProfile();
  return {
    name: 'site-seo',
    transformIndexHtml: (html) => applySeo(html, profile),
    generateBundle() {
      this.emitFile({ type: 'asset', fileName: 'robots.txt', source: robotsTxt(profile) });
      this.emitFile({ type: 'asset', fileName: 'sitemap.xml', source: sitemapXml(profile) });
    },
  };
};

// `base: './'` + hash routing means the same build works from any URL:
// a domain root, a GitHub Pages project path (/repo-name/), Netlify, Vercel, S3...
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), 'VITE_');
  const storesOnly = env.VITE_SITE_MODE === 'stores';
  return {
    base: './',
    plugins: [react(), storesOnly ? neutralHtml() : seo()],
    // src/main.jsx imports 'app-entry', which is the whole site or only the stores.
    resolve: { alias: { 'app-entry': here(storesOnly ? './src/StoresApp.jsx' : './src/App.jsx') } },
    // The stores build ships none of the main site's static files (its logo and icon).
    publicDir: storesOnly ? false : 'public',
    build: {
      target: 'es2020',
      chunkSizeWarningLimit: 700,
    },
  };
});
