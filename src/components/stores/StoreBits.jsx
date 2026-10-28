import { useState } from 'react';
import { LuStore } from 'react-icons/lu';
import { Button, Empty } from '../ui';
import { config } from '../../config/env';
import { inkOn, safeHex } from '../../lib/color';

/** Shown on every store page when this site has not been given the Store backend's key. */
export function StoresOff() {
  return (
    <div className="container page page--narrow">
      <Empty icon={LuStore} title="Stores are not switched on yet" action={<Button to="/" size="sm">Back to home</Button>}>
        This site has not been connected to its store backend. If you run the site, set VITE_STORES_SUPABASE_ANON_KEY (see the README) and rebuild.
      </Empty>
    </div>
  );
}

/** A store's logo, or its first letter on its accent colour. */
export function StoreAvatar({ store, size = 'md' }) {
  const [broken, setBroken] = useState(false);
  const accent = safeHex(store?.accent);
  const initial = (store?.name || '?').trim().charAt(0).toUpperCase();
  return (
    <span className={`avatar avatar--${size}`} style={{ '--av': accent, '--av-ink': inkOn(accent) }} aria-hidden="true">
      {store?.logo && !broken ? <img src={store.logo} alt="" loading="lazy" onError={() => setBroken(true)} /> : initial}
    </span>
  );
}

/** Wraps a page so it shows the "not switched on" notice when the Store backend has no key. */
export const withStores = (Page) =>
  function StoresGate(props) {
    return config.stores.enabled ? <Page {...props} /> : <StoresOff />;
  };
