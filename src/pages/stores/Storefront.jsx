import { useEffect, useMemo, useRef, useState } from 'react';
import { LuCircleAlert, LuMessageCircle, LuRefreshCw, LuShoppingBag, LuStore } from 'react-icons/lu';
import { Button, Callout, CopyButton, Empty, NetworkChip, Segmented } from '../../components/ui';
import StoreCheckout from '../../components/stores/StoreCheckout';
import { StoreAvatar, withStores } from '../../components/stores/StoreBits';
import { useAsync } from '../../hooks/useAsync';
import { useDocTitle } from '../../hooks/useDocTitle';
import { useTintColor } from '../../hooks/useTintColor';
import { NETWORKS } from '../../config/site';
import { money, splitSize } from '../../lib/format';
import { shopPath, storeUrl, whatsappLink } from '../../lib/links';
import { fetchBundlesByStore, fetchStoreBySlug } from '../../services/stores';

/**
 * A store's own page: its bundles and a checkout. `param` is the slug from the address.
 * In the customer view (`shell`) the store is passed in already loaded and the page has no way back to the rest of the site.
 */
export function Storefront({ param, store: preloaded, shell = false }) {
  const slug = preloaded?.slug || param;
  const { data, status, reload } = useAsync(async () => {
    const store = preloaded || (await fetchStoreBySlug(slug));
    if (!store) return { store: null, bundles: [] };
    const bundles = await fetchBundlesByStore(store.id);
    return { store, bundles: bundles.map((b) => ({ ...b, store })) };
  }, [slug]);
  const store = data?.store;
  useDocTitle(store ? store.name : 'Store');
  useTintColor(store?.accent);

  const [network, setNetwork] = useState('All');
  const [pickedId, setPickedId] = useState(null);
  const checkoutRef = useRef(null);

  const bundles = useMemo(() => data?.bundles || [], [data]);
  const networks = useMemo(() => NETWORKS.filter((n) => bundles.some((b) => b.network === n.id)), [bundles]);
  const shown = bundles.filter((b) => network === 'All' || b.network === network);
  const picked = bundles.find((b) => String(b.id) === String(pickedId));

  useEffect(() => {
    setNetwork('All');
    setPickedId(null);
  }, [slug]);

  const pick = (id) => {
    setPickedId(id);
    // On a phone the checkout sits below the list: bring it into view.
    if (window.matchMedia('(max-width: 899px)').matches) {
      const calm = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
      setTimeout(() => checkoutRef.current?.scrollIntoView({ behavior: calm ? 'auto' : 'smooth', block: 'start' }), 60);
    }
  };

  if (status === 'loading' && !data) {
    return <div className="container page"><div className="skeleton-block" aria-busy="true" /></div>;
  }
  if (status === 'error') {
    return (
      <div className="container page page--narrow">
        <Callout tone="bad" icon={LuCircleAlert}>
          <p>We couldn’t load this store. Check your connection.</p>
          <Button variant="secondary" size="sm" onClick={reload}><LuRefreshCw aria-hidden="true" /> Try again</Button>
        </Callout>
      </div>
    );
  }
  if (!store) {
    return (
      <div className="container page page--narrow">
        <Empty icon={LuStore} title="Store not found" action={shell ? undefined : <Button to="/stores" size="sm">Browse stores</Button>}>
          The link may be old, or the store may have closed.
        </Empty>
      </div>
    );
  }

  const chat = whatsappLink(store.whatsapp, `Hi ${store.name}, I have a question about a data bundle.`);

  return (
    <div className="container page storepage">
      <header className="storehead">
        <StoreAvatar store={store} size="xl" />
        <div className="storehead__text">
          <h1 className="storehead__name">{store.name}</h1>
          {store.description && <p className="storehead__about">{store.description}</p>}
        </div>
        <div className="storehead__actions">
          {chat && <Button href={chat} target="_blank" rel="noreferrer" variant="secondary" size="sm"><LuMessageCircle aria-hidden="true" /> WhatsApp</Button>}
          <CopyButton text={storeUrl(store.slug)} label="Copy link" />
        </div>
      </header>

      <div className="buy">
        <div className="buy__steps">
          <section className="step" aria-labelledby="store-bundles">
            <h2 id="store-bundles" className="step__title"><span className="step__n">1</span> Choose a bundle</h2>
            {bundles.length === 0 ? (
              <Empty icon={LuShoppingBag} title="Nothing on sale right now">This store has not listed any bundles yet. Check back soon.</Empty>
            ) : (
              <>
                {networks.length > 1 && (
                  <div className="storefilter">
                    <Segmented label="Network" value={network} onChange={setNetwork} options={[{ value: 'All', label: 'All' }, ...networks.map((n) => ({ value: n.id, label: n.label }))]} />
                  </div>
                )}
                <div className="bundlegrid" role="group" aria-label="Bundles">
                  {shown.map((b) => {
                    const { num, unit } = splitSize(b.size);
                    const on = String(pickedId) === String(b.id);
                    return (
                      <button key={b.id} type="button" className={`bundle${on ? ' is-on' : ''}`} aria-pressed={on} onClick={() => pick(b.id)}>
                        <span className="bundle__size">{num}<span className="bundle__unit">{unit}</span></span>
                        <span className="bundle__price">{money(b.price)}</span>
                        <span className="bundle__meta"><NetworkChip network={b.network} /> {b.validity}</span>
                      </button>
                    );
                  })}
                </div>
              </>
            )}
          </section>
        </div>

        <aside className="buy__summary" aria-label="Checkout" ref={checkoutRef}>
          {picked ? (
            <StoreCheckout bundle={picked} />
          ) : (
            <div className="panel checkout-idle">
              <LuShoppingBag aria-hidden="true" />
              <p>Choose a bundle to see your order and pay.</p>
            </div>
          )}
        </aside>
      </div>
    </div>
  );
}

export default /* @__PURE__ */ withStores(Storefront);
