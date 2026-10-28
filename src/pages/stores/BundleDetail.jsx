import { LuCircleAlert, LuRefreshCw, LuStore } from 'react-icons/lu';
import { Button, Callout, Empty, NetworkChip } from '../../components/ui';
import StoreCheckout from '../../components/stores/StoreCheckout';
import { StoreAvatar, withStores } from '../../components/stores/StoreBits';
import { useAsync } from '../../hooks/useAsync';
import { usePageTitle } from '../../hooks/usePageTitle';
import { useTintColor } from '../../hooks/useTintColor';
import { Link } from '../../router';
import { money, splitSize } from '../../lib/format';
import { fetchBundleById } from '../../services/stores';

/** One bundle from the marketplace. `param` is its id from the address. */
function BundleDetail({ param: id }) {
  const { data: bundle, status, reload } = useAsync(() => fetchBundleById(id), [id]);
  usePageTitle(bundle ? `${bundle.size} ${bundle.network}` : 'Bundle');
  useTintColor(bundle?.store?.accent);

  if (status === 'loading' && !bundle) {
    return <div className="container page"><div className="skeleton-block" aria-busy="true" /></div>;
  }
  if (status === 'error') {
    return (
      <div className="container page page--narrow">
        <Callout tone="bad" icon={LuCircleAlert}>
          <p>We couldn’t load this bundle. Check your connection.</p>
          <Button variant="secondary" size="sm" onClick={reload}><LuRefreshCw aria-hidden="true" /> Try again</Button>
        </Callout>
      </div>
    );
  }
  if (!bundle) {
    return (
      <div className="container page page--narrow">
        <Empty icon={LuStore} title="Bundle not found" action={<Button to="/marketplace" size="sm">Back to the marketplace</Button>}>
          It may have been removed or hidden by the store.
        </Empty>
      </div>
    );
  }

  const { num, unit } = splitSize(bundle.size);
  const store = bundle.store;

  return (
    <div className="container page storepage">
      <p className="crumbs"><Link to="/marketplace">Marketplace</Link> <span aria-hidden="true">/</span> <span>{bundle.size} {bundle.network}</span></p>
      <div className="buy">
        <div className="buy__steps">
          <section className="panel bundlehero">
            <NetworkChip network={bundle.network} long />
            <h1 className="bundlehero__size">{num}<span className="bundle__unit">{unit}</span></h1>
            <p className="bundlehero__price">{money(bundle.price)}</p>
            <dl className="summary">
              <div><dt>Bundle</dt><dd>{bundle.name}</dd></div>
              <div><dt>Valid for</dt><dd>{bundle.validity}</dd></div>
              <div><dt>Delivered by</dt><dd>{store.name}</dd></div>
            </dl>
          </section>
          <Link to={`/store/${encodeURIComponent(store.slug)}`} className="panel soldby">
            <StoreAvatar store={store} size="lg" />
            <span className="soldby__text">
              <span className="soldby__label">Sold by</span>
              <strong>{store.name}</strong>
              {store.description && <span>{store.description}</span>}
            </span>
            <span className="soldby__go">See all bundles →</span>
          </Link>
        </div>
        <aside className="buy__summary" aria-label="Checkout">
          <StoreCheckout bundle={bundle} />
        </aside>
      </div>
    </div>
  );
}

export default withStores(BundleDetail);
