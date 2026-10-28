import { useEffect } from 'react';
import { LuMessageCircle } from 'react-icons/lu';
import { Link, NavLink } from '../../router';
import { ThemeToggle } from '../ThemeToggle';
import { StoreAvatar } from './StoreBits';
import { inkOn, safeHex } from '../../lib/color';
import { shopPath, whatsappLink } from '../../lib/links';

const escapeXml = (s) => String(s).replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&apos;' })[c]);

/** A square icon for the browser tab: the store's logo, or its first letter on its colour. */
function tabIcon(store) {
  if (store.logo && /^https?:\/\//i.test(store.logo)) return { href: store.logo, type: '' };
  const accent = safeHex(store.accent);
  const initial = escapeXml((store.name || '?').trim().charAt(0).toUpperCase() || '?');
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><rect width="32" height="32" rx="9" fill="${accent}"/><text x="16" y="23" font-size="19" font-weight="700" text-anchor="middle" fill="${inkOn(accent)}" font-family="sans-serif">${initial}</text></svg>`;
  return { href: `data:image/svg+xml,${encodeURIComponent(svg)}`, type: 'image/svg+xml' };
}

/**
 * Keeps the browser tab about the store only: its name as the title and its logo as the icon.
 * Until the store has loaded the tab is blank, so the site's own name never shows to a customer.
 */
function useShopIdentity(store, view) {
  useEffect(() => {
    const head = document.head;
    let link = head.querySelector('link[rel="icon"]');
    const made = !link;
    if (!link) {
      link = document.createElement('link');
      link.rel = 'icon';
      head.appendChild(link);
    }
    const before = { title: document.title, href: link.getAttribute('href'), type: link.getAttribute('type') };
    const set = (href, type) => {
      link.setAttribute('href', href);
      if (type) link.setAttribute('type', type);
      else link.removeAttribute('type');
    };
    document.title = store ? (view === 'track' ? `Track your order | ${store.name}` : store.name) : 'Shop';
    if (store) {
      const icon = tabIcon(store);
      set(icon.href, icon.type);
    } else {
      set('data:,', '');
    }
    return () => {
      document.title = before.title;
      if (made) link.remove();
      else set(before.href || '', before.type || '');
    };
  }, [store, view]);
}

/**
 * The only frame a customer of a store ever sees: the store's name, a link to shop and a link to track an order.
 * It deliberately has no link to the rest of the site.
 */
export default function ShopLayout({ store, slug, view, children }) {
  useShopIdentity(store, view);
  const base = shopPath(store?.slug || slug);
  const chat = store ? whatsappLink(store.whatsapp, `Hi ${store.name}, I have a question.`) : '';

  return (
    <div className="site shop">
      <a href="#main" className="skip" onClick={(e) => { e.preventDefault(); document.getElementById('main')?.focus(); }}>
        Skip to content
      </a>
      <header className="header">
        <div className="container header__bar">
          <Link to={base} className="header__brand shopbrand">
            {store && <StoreAvatar store={store} size="sm" />}
            <span className="shopbrand__name">{store?.name || ''}</span>
          </Link>
          <nav className="shopnav" aria-label="Store">
            <NavLink to={base} end className="header__link">Shop</NavLink>
            <NavLink to={`${base}/track`} className="header__link">Track order</NavLink>
          </nav>
          <div className="header__actions">
            <ThemeToggle />
          </div>
        </div>
      </header>
      <main id="main" tabIndex={-1} className="site__main">
        {children}
      </main>
      <footer className="shopfoot">
        <div className="container shopfoot__row">
          <p>{store ? <>© {new Date().getFullYear()} {store.name}</> : ''}</p>
          {chat && (
            <a href={chat} target="_blank" rel="noreferrer" className="shopfoot__chat">
              <LuMessageCircle aria-hidden="true" /> Chat with the store on WhatsApp
            </a>
          )}
        </div>
      </footer>
    </div>
  );
}
