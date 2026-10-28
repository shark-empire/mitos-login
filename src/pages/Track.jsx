import { useEffect, useRef, useState } from 'react';
import { LuClipboardPaste, LuMessageCircle, LuPackage, LuSearch } from 'react-icons/lu';
import { Button, Empty, NetworkChip, PageHeader, StatusBadge, TextField } from '../components/ui';
import { useToast } from '../context/Toast';
import { usePageTitle } from '../hooks/usePageTitle';
import { Link, useLocation } from '../router';
import { findOrdersByReference } from '../services/webhooks';
import { recentRefs } from '../lib/storage';
import { dateTime, moneyFromPesewas } from '../lib/format';
import { normalizeNetwork, site } from '../config/site';

export default function Track() {
  usePageTitle('Track order');
  const toast = useToast();
  const { search } = useLocation();
  const initial = search.get('ref') || '';
  const [ref, setRef] = useState(initial);
  const [state, setState] = useState({ phase: 'idle', orders: [], message: '' });
  const [recent] = useState(() => recentRefs.list());
  const searched = useRef('');

  const lookup = async (value = ref) => {
    const reference = value.trim();
    if (!reference) {
      toast.err('Enter a payment reference first.');
      return;
    }
    searched.current = reference;
    setState({ phase: 'loading', orders: [], message: '' });
    try {
      const res = await findOrdersByReference(reference);
      if (searched.current !== reference) return;
      if (res.error) setState({ phase: 'error', orders: [], message: String(res.error) });
      else if (!res.orders?.length) setState({ phase: 'empty', orders: [], message: reference });
      else setState({ phase: 'found', orders: res.orders, message: '' });
    } catch (e) {
      if (searched.current === reference) setState({ phase: 'error', orders: [], message: e.message });
    }
  };

  useEffect(() => {
    if (initial) lookup(initial);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const paste = async () => {
    try {
      const text = (await navigator.clipboard.readText()).trim();
      setRef(text);
    } catch {
      toast.info('Allow clipboard access to paste, or type the reference.');
    }
  };

  return (
    <div className="container page page--narrow">
      <PageHeader title="Track order" lead="Enter the reference you received when you paid. It starts with SF-." />

      <form
        className="trackform"
        onSubmit={(e) => {
          e.preventDefault();
          lookup();
        }}
      >
        <TextField
          label="Payment reference"
          placeholder="SF-1718123456-482910"
          value={ref}
          onChange={(e) => setRef(e.target.value)}
          autoComplete="off"
          autoCapitalize="off"
          spellCheck={false}
          suffix={
            <button type="button" className="field__action" onClick={paste}>
              <LuClipboardPaste aria-hidden="true" /> Paste
            </button>
          }
        />
        <Button type="submit" size="lg" loading={state.phase === 'loading'}>
          <LuSearch aria-hidden="true" /> Find order
        </Button>
      </form>

      {recent.length > 0 && (
        <div className="recent">
          <span className="recent__label">Recent on this device</span>
          <div className="recent__list">
            {recent.map((r) => (
              <button key={r} type="button" className="recent__chip" onClick={() => { setRef(r); lookup(r); }}>{r}</button>
            ))}
          </div>
        </div>
      )}

      <div className="results" aria-live="polite">
        {state.phase === 'loading' && <div className="order order--skeleton" aria-hidden="true" />}

        {state.phase === 'empty' && (
          <Empty icon={LuPackage} title="No order found" action={<Button variant="secondary" size="sm" href={site.whatsappUrl(`Hi, I can't find my order ${state.message}`)} target="_blank" rel="noreferrer"><LuMessageCircle aria-hidden="true" /> Ask support</Button>}>
            We couldn’t find <strong>{state.message}</strong>. Check the reference, or message us if you paid a while ago.
          </Empty>
        )}

        {state.phase === 'error' && (
          <Empty icon={LuPackage} title="We couldn’t check that order">{state.message || 'Try again in a moment.'}</Empty>
        )}

        {state.phase === 'found' && (
          <>
            <p className="results__count">{state.orders.length} order{state.orders.length === 1 ? '' : 's'} found</p>
            <ul className="orderlist">
              {state.orders.map((o, i) => {
                const reference = o.reference || '';
                return (
                  <li key={reference || i} className="order">
                    <div className="order__top">
                      <div>
                        <code className="order__ref">{reference || 'N/A'}</code>
                        <span className="order__time">{dateTime(o.created_at) || 'Unknown time'}</span>
                      </div>
                      <StatusBadge status={o.status} />
                    </div>
                    <dl className="order__meta">
                      <div><dt>Network</dt><dd>{o.network ? <NetworkChip network={normalizeNetwork(o.network)} /> : 'N/A'}</dd></div>
                      <div><dt>Bundle</dt><dd>{o.bundle_size || o.volume || 'N/A'}</dd></div>
                      <div><dt>Phone</dt><dd>{o.phone || 'N/A'}</dd></div>
                      <div><dt>Amount paid</dt><dd>{o.amount ? moneyFromPesewas(o.amount) : 'N/A'}</dd></div>
                    </dl>
                    <div className="order__actions">
                      <Button variant="secondary" size="sm" href={site.whatsappUrl(`Hi, I need help with order ${reference}`)} target="_blank" rel="noreferrer"><LuMessageCircle aria-hidden="true" /> Get help</Button>
                      <Button variant="ghost" size="sm" to="/data">Buy again</Button>
                    </div>
                  </li>
                );
              })}
            </ul>
          </>
        )}
      </div>

      <p className="status__fine">Have an account? <Link to="/account/orders">See all your orders</Link>.</p>
    </div>
  );
}
