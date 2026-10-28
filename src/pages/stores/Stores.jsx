import { useMemo, useState } from 'react';
import { LuCircleAlert, LuRefreshCw, LuStore } from 'react-icons/lu';
import { Button, Callout, Empty, PageHeader, TextField } from '../../components/ui';
import { StoreAvatar, withStores } from '../../components/stores/StoreBits';
import { useAsync } from '../../hooks/useAsync';
import { usePageTitle } from '../../hooks/usePageTitle';
import { Link } from '../../router';
import { fetchStores } from '../../services/stores';

function Stores() {
  usePageTitle('Stores');
  const { data, status, reload } = useAsync(fetchStores);
  const [q, setQ] = useState('');
  const list = useMemo(() => {
    const term = q.trim().toLowerCase();
    return (data || []).filter((s) => !term || `${s.name} ${s.description}`.toLowerCase().includes(term));
  }, [data, q]);

  return (
    <div className="container page">
      <PageHeader
        title="Stores"
        lead="Each store sets its own prices and delivers the data itself."
        actions={<Button to="/store-builder" size="sm">Open your store</Button>}
      />
      <div className="toolbar">
        <TextField className="toolbar__search" label="Search stores" type="search" placeholder="Store name" value={q} onChange={(e) => setQ(e.target.value)} />
      </div>

      {status === 'loading' && !data && (
        <div className="storegrid" aria-busy="true">
          {Array.from({ length: 6 }).map((_, i) => <div key={i} className="storecard storecard--skeleton" />)}
        </div>
      )}
      {status === 'error' && (
        <Callout tone="bad" icon={LuCircleAlert}>
          <p>We couldn’t load the stores. Check your connection.</p>
          <Button variant="secondary" size="sm" onClick={reload}><LuRefreshCw aria-hidden="true" /> Try again</Button>
        </Callout>
      )}
      {status !== 'error' && data && list.length === 0 && (
        <Empty icon={LuStore} title={data.length === 0 ? 'No stores yet' : 'No stores match'}>
          {data.length === 0 ? 'Be the first to open one.' : 'Try a different search.'}
        </Empty>
      )}
      {list.length > 0 && (
        <div className="storegrid">
          {list.map((s) => (
            <Link key={s.id} to={`/store/${encodeURIComponent(s.slug)}`} className="storecard" style={{ '--sc': s.accent }}>
              <StoreAvatar store={s} size="lg" />
              <span className="storecard__name">{s.name}</span>
              <span className="storecard__text">{s.description || 'Data bundles at fair prices.'}</span>
              <span className="storecard__go">Visit store →</span>
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}

export default withStores(Stores);
