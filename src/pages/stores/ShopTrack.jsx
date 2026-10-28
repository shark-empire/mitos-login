import { useEffect, useRef, useState } from 'react';
import { LuClipboardPaste, LuMessageCircle, LuPackage, LuRefreshCw, LuSearch } from 'react-icons/lu';
import { Button, Empty, NetworkChip, PageHeader, StatusBadge, TextField } from '../../components/ui';
import { useToast } from '../../context/Toast';
import { useLocation } from '../../router';
import { trackOrder } from '../../services/stores';
import { shopOrders } from '../../lib/storage';
import { dateTime, money } from '../../lib/format';
import { shopPath, whatsappLink } from '../../lib/links';

/** What to tell the customer, by order status. */
function progress(status, store) {
  const s = String(status || '').toLowerCase();
  if (['completed', 'delivered', 'success'].includes(s)) return 'Delivered. Your data has been sent. If it has not shown up on your phone, message the store.';
  if (s === 'failed') return `This order could not be completed. Message ${store.name} with your reference and they will sort it out.`;
  return `We have your payment and ${store.name} is sending your bundle. Most orders arrive within a few minutes.`;
}

/** A store's order tracker: the customer pastes the reference from their receipt. */
export default function ShopTrack({ store }) {
  const toast = useToast();
  const { search } = useLocation();
  const initial = search.get('ref') || '';
  const [ref, setRef] = useState(initial);
  const [state, setState] = useState({ phase: 'idle', order: null, message: '' });
  const [recent] = useState(() => shopOrders.list(store.slug));
  const searched = useRef('');

  const lookup = async (value = ref) => {
    const reference = value.trim();
    if (!reference) {
      toast.err('Enter the payment reference first.');
      return;
    }
    searched.current = reference;
    setState({ phase: 'loading', order: null, message: '' });
    try {
      const order = await trackOrder(reference);
      if (searched.current !== reference) return;
      // An order from another store is treated as not found: this page is for this store's customers only.
      if (!order || (order.storeSlug && order.storeSlug !== store.slug)) setState({ phase: 'empty', order: null, message: reference });
      else setState({ phase: 'found', order, message: '' });
    } catch {
      if (searched.current === reference) setState({ phase: 'error', order: null, message: '' });
    }
  };

  useEffect(() => {
    if (initial) lookup(initial);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const paste = async () => {
    try {
      setRef((await navigator.clipboard.readText()).trim());
    } catch {
      toast.info('Allow clipboard access to paste, or type the reference.');
    }
  };

  const help = (reference) => whatsappLink(store.whatsapp, `Hi, I need help with my order. Reference: ${reference}`);
  const o = state.order;

  return (
    <div className="container page page--narrow">
      <PageHeader title="Track your order" lead="Enter the reference you got when you paid. It starts with DP-." />

      <form className="trackform" onSubmit={(e) => { e.preventDefault(); lookup(); }}>
        <TextField
          label="Payment reference"
          placeholder="DP-1718123456789-AB12"
          value={ref}
          onChange={(e) => setRef(e.target.value)}
          autoComplete="off"
          autoCapitalize="off"
          spellCheck={false}
          suffix={<button type="button" className="field__action" onClick={paste}><LuClipboardPaste aria-hidden="true" /> Paste</button>}
        />
        <Button type="submit" size="lg" loading={state.phase === 'loading'}><LuSearch aria-hidden="true" /> Find order</Button>
      </form>

      {recent.length > 0 && (
        <div className="recent">
          <span className="recent__label">Your orders on this device</span>
          <div className="recent__list">
            {recent.map((r) => (
              <button key={r.ref} type="button" className="recent__chip" onClick={() => { setRef(r.ref); lookup(r.ref); }}>
                {r.size ? `${r.size} ${r.network} · ` : ''}{r.ref}
              </button>
            ))}
          </div>
        </div>
      )}

      <div className="results" aria-live="polite">
        {state.phase === 'loading' && <div className="order order--skeleton" aria-hidden="true" />}

        {state.phase === 'empty' && (
          <Empty
            icon={LuPackage}
            title="No order found"
            action={help(state.message) && <Button variant="secondary" size="sm" href={help(state.message)} target="_blank" rel="noreferrer"><LuMessageCircle aria-hidden="true" /> Ask the store</Button>}
          >
            We couldn’t find <strong>{state.message}</strong> for this store. Check the reference. If you only just paid, wait a minute and try again.
          </Empty>
        )}

        {state.phase === 'error' && <Empty icon={LuPackage} title="We couldn’t check that order">Check your connection and try again in a moment.</Empty>}

        {state.phase === 'found' && o && (
          <ul className="orderlist">
            <li className="order">
              <div className="order__top">
                <div>
                  <code className="order__ref">{o.reference}</code>
                  <span className="order__time">{dateTime(o.createdAt) || ''}</span>
                </div>
                <StatusBadge status={o.status} />
              </div>
              <dl className="order__meta">
                <div><dt>Network</dt><dd>{o.network ? <NetworkChip network={o.network} /> : '–'}</dd></div>
                <div><dt>Bundle</dt><dd>{o.size || '–'}</dd></div>
                <div><dt>Sent to</dt><dd>{o.phone || '–'}</dd></div>
                <div><dt>Amount paid</dt><dd>{o.amount ? money(o.amount) : '–'}</dd></div>
              </dl>
              <p className="order__note">{progress(o.status, store)}</p>
              <div className="order__actions">
                <Button variant="secondary" size="sm" onClick={() => lookup(o.reference)}><LuRefreshCw aria-hidden="true" /> Check again</Button>
                {help(o.reference) && <Button variant="secondary" size="sm" href={help(o.reference)} target="_blank" rel="noreferrer"><LuMessageCircle aria-hidden="true" /> Get help</Button>}
                <Button variant="ghost" size="sm" to={shopPath(store.slug)}>Buy again</Button>
              </div>
            </li>
          </ul>
        )}
      </div>
    </div>
  );
}
