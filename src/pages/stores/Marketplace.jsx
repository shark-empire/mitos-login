import { useMemo, useState } from 'react';
import { LuCircleAlert, LuRefreshCw, LuStore } from 'react-icons/lu';
import { Button, Callout, Empty, NetworkChip, PageHeader, Segmented, TextField } from '../../components/ui';
import { StoreAvatar, withStores } from '../../components/stores/StoreBits';
import { useAsync } from '../../hooks/useAsync';
import { usePageTitle } from '../../hooks/usePageTitle';
import { useTint } from '../../hooks/useTint';
import { Link } from '../../router';
import { NETWORKS } from '../../config/site';
import { money, splitSize } from '../../lib/format';
import { fetchBundles } from '../../services/stores';

const SORTS = [
  { value: 'new', label: 'Newest' },
  { value: 'price', label: 'Price: low to high' },
  { value: 'size', label: 'Size: large to small' },
];

/** One bundle on sale, as a card that opens its page. */
export function BundleCard({ bundle }) {
  const { num, unit } = splitSize(bundle.size);
  return (
    <Link to={`/bundle/${encodeURIComponent(bundle.id)}`} className="mbundle">
      <span className="mbundle__top">
        <NetworkChip network={bundle.network} />
        <span className="mbundle__valid">{bundle.validity}</span>
      </span>
      <span className="mbundle__size">{num}<span className="mbundle__unit">{unit}</span></span>
      <span className="mbundle__price">{money(bundle.price)}</span>
      {bundle.store && (
        <span className="mbundle__store">
          <StoreAvatar store={bundle.store} size="xs" />
          <span>{bundle.store.name}</span>
        </span>
      )}
    </Link>
  );
}

function Marketplace() {
  usePageTitle('Marketplace');
  const { data, status, reload } = useAsync(fetchBundles);
  const [network, setNetwork] = useState('All');
  const [q, setQ] = useState('');
  const [sort, setSort] = useState('new');
  useTint(NETWORKS.find((n) => n.id === network)?.meta);

  const list = useMemo(() => {
    const term = q.trim().toLowerCase();
    const out = (data || []).filter(
      (b) =>
        (network === 'All' || b.network === network) &&
        (!term || `${b.name} ${b.size} ${b.network} ${b.store?.name || ''}`.toLowerCase().includes(term)),
    );
    if (sort === 'price') out.sort((a, b) => a.price - b.price);
    if (sort === 'size') out.sort((a, b) => b.sizeGb - a.sizeGb);
    return out;
  }, [data, network, q, sort]);

  return (
    <div className="container page">
      <PageHeader
        title="Marketplace"
        lead="Data bundles from independent stores. Pick one, pay with Mobile Money or card, and the store delivers it."
        actions={<Button to="/stores" variant="secondary" size="sm"><LuStore aria-hidden="true" /> Browse stores</Button>}
      />

      <div className="toolbar">
        <Segmented
          label="Network"
          value={network}
          onChange={setNetwork}
          options={[{ value: 'All', label: 'All' }, ...NETWORKS.map((n) => ({ value: n.id, label: n.label }))]}
        />
        <TextField className="toolbar__search" label="Search" type="search" placeholder="Bundle or store name" value={q} onChange={(e) => setQ(e.target.value)} />
        <TextField className="toolbar__sort" label="Sort by" as="select" value={sort} onChange={(e) => setSort(e.target.value)}>
          {SORTS.map((s) => <option key={s.value} value={s.value}>{s.label}</option>)}
        </TextField>
      </div>

      {status === 'loading' && !data && (
        <div className="mgrid" aria-busy="true">
          {Array.from({ length: 8 }).map((_, i) => <div key={i} className="mbundle mbundle--skeleton" />)}
        </div>
      )}
      {status === 'error' && (
        <Callout tone="bad" icon={LuCircleAlert}>
          <p>We couldn’t load the marketplace. Check your connection.</p>
          <Button variant="secondary" size="sm" onClick={reload}><LuRefreshCw aria-hidden="true" /> Try again</Button>
        </Callout>
      )}
      {status !== 'error' && data && list.length === 0 && (
        <Empty icon={LuStore} title={data.length === 0 ? 'Nothing on sale yet' : 'No bundles match'} action={data.length === 0 ? <Button to="/store-builder" size="sm">Open the first store</Button> : undefined}>
          {data.length === 0 ? 'Stores add their bundles here as soon as they open.' : 'Try another network or a different search.'}
        </Empty>
      )}
      {data && list.length > 0 && (
        <>
          <p className="results__count" aria-live="polite">{list.length} {list.length === 1 ? 'bundle' : 'bundles'}</p>
          <div className="mgrid">
            {list.map((b) => <BundleCard key={b.id} bundle={b} />)}
          </div>
        </>
      )}
    </div>
  );
}

export default withStores(Marketplace);
