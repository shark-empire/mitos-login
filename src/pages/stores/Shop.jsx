import { LuCircleAlert, LuRefreshCw, LuStore } from 'react-icons/lu';
import { Button, Callout, Empty } from '../../components/ui';
import ShopLayout from '../../components/stores/ShopLayout';
import { Storefront } from './Storefront';
import ShopTrack from './ShopTrack';
import { useAsync } from '../../hooks/useAsync';
import { useTintColor } from '../../hooks/useTintColor';
import { config } from '../../config/env';
import { fetchStoreBySlug } from '../../services/stores';

const Notice = ({ title, children, action }) => (
  <div className="container page page--narrow">
    <Empty icon={LuStore} title={title} action={action}>{children}</Empty>
  </div>
);

/** One store as its customers see it: the shop or the order tracker, inside the store-only frame. */
export default function Shop({ slug, view }) {
  const { data: store, status, reload } = useAsync(() => (config.stores.enabled ? fetchStoreBySlug(slug) : Promise.resolve(null)), [slug]);
  useTintColor(store?.accent);

  let body;
  if (!config.stores.enabled) {
    body = <Notice title="This store isn’t available right now">Please try again later.</Notice>;
  } else if (status === 'loading' && !store) {
    body = <div className="container page"><div className="skeleton-block" aria-busy="true" /></div>;
  } else if (status === 'error') {
    body = (
      <div className="container page page--narrow">
        <Callout tone="bad" icon={LuCircleAlert}>
          <p>We couldn’t load this store. Check your connection.</p>
          <Button variant="secondary" size="sm" onClick={reload}><LuRefreshCw aria-hidden="true" /> Try again</Button>
        </Callout>
      </div>
    );
  } else if (!store) {
    body = <Notice title="Store not found">The link may be old, or the store may have closed.</Notice>;
  } else if (view === 'track') {
    body = <ShopTrack store={store} />;
  } else {
    body = <Storefront store={store} shell />;
  }

  return (
    <ShopLayout store={store} slug={slug} view={view}>
      {body}
    </ShopLayout>
  );
}
