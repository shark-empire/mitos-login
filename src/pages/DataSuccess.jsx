import { useCallback, useEffect, useRef, useState } from 'react';
import { LuCircleAlert, LuCircleCheck, LuClock, LuRadioTower } from 'react-icons/lu';
import { Button, CopyButton, NetworkChip, Spinner, Ticket } from '../components/ui';
import { useToast } from '../context/Toast';
import { usePageTitle } from '../hooks/usePageTitle';
import { Link, useLocation } from '../router';
import { dataApi, accountsApi } from '../services/rest';
import { useAuth } from '../context/Auth';
import { recentRefs } from '../lib/storage';
import { moneyFromPesewas, money } from '../lib/format';
import { normalizeNetwork, site } from '../config/site';

const POLL_MS = 3000;
const MAX_POLLS = 20;

/**
 * Waits for the payment automation to record the order, then shows the receipt.
 *  - ?ref=SF-...    guest / Paystack purchases: polls `transactions` on the data backend
 *  - ?order=SF...   wallet purchases: polls `orders` on the accounts backend
 */
export default function DataSuccess() {
  usePageTitle('Payment status');
  const { search } = useLocation();
  const ref = search.get('ref');
  const orderId = search.get('order');
  const toast = useToast();
  const { authed } = useAuth();

  const [state, setState] = useState(ref || orderId ? 'checking' : 'missing');
  const [attempt, setAttempt] = useState(0);
  const [receipt, setReceipt] = useState(null);
  const [runId, setRunId] = useState(0);
  const timer = useRef(null);

  const identifier = ref || orderId;

  const check = useCallback(
    async (n, cancelled) => {
      try {
        let found = null;
        if (ref) {
          const rows = await dataApi.select(
            'transactions',
            `?select=phone,bundle_id,network,amount,created_at&paystack_ref=eq.${encodeURIComponent(ref)}&limit=1`,
          );
          if (rows?.[0]) {
            const r = rows[0];
            found = { phone: r.phone, bundle: r.bundle_id, network: normalizeNetwork(r.network), amount: moneyFromPesewas(r.amount), status: 'Paid' };
          }
        } else {
          const rows = await authed((t) =>
            accountsApi.select('orders', `?order_id=eq.${encodeURIComponent(orderId)}&select=*&limit=1`, t),
          );
          if (rows?.[0]) {
            const r = rows[0];
            found = { phone: r.phone || r.recipient_phone, bundle: r.size || r.bundle_id, network: r.network ? normalizeNetwork(r.network) : null, amount: money(r.amount), status: r.status };
          }
        }
        if (cancelled.current) return;
        if (found) {
          setReceipt(found);
          setState('done');
          if (String(found.status).toLowerCase() === 'completed') toast.ok('Your data has been delivered.');
          return;
        }
      } catch {
        /* keep polling: the record may simply not exist yet */
      }
      if (cancelled.current) return;
      if (n >= MAX_POLLS) {
        setState('timeout');
        return;
      }
      setAttempt(n);
      timer.current = setTimeout(() => check(n + 1, cancelled), POLL_MS);
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [ref, orderId, authed],
  );

  useEffect(() => {
    if (!identifier) return undefined;
    if (ref) recentRefs.add(ref);
    const cancelled = { current: false };
    setState('checking');
    setAttempt(0);
    check(1, cancelled);
    return () => {
      cancelled.current = true;
      clearTimeout(timer.current);
    };
  }, [identifier, ref, runId, check]);

  return (
    <div className="container page page--narrow">
      <div className="status">
        {state === 'checking' && (
          <>
            <div className="status__icon status__icon--busy"><Spinner size={34} /></div>
            <h1 className="status__title">Confirming your payment</h1>
            <p className="status__text">This usually takes a few seconds. Keep this page open.</p>
            {attempt > 3 && <p className="status__fine">Check {attempt} of {MAX_POLLS}</p>}
          </>
        )}

        {state === 'missing' && (
          <>
            <div className="status__icon status__icon--bad"><LuCircleAlert size={34} /></div>
            <h1 className="status__title">No payment reference found</h1>
            <p className="status__text">Open this page from your payment confirmation, or look up an order with its reference.</p>
            <div className="status__actions">
              <Button to="/track">Track an order</Button>
              <Button to="/data" variant="secondary">Buy data</Button>
            </div>
          </>
        )}

        {state === 'timeout' && (
          <>
            <div className="status__icon status__icon--warn"><LuClock size={34} /></div>
            <h1 className="status__title">We’re still waiting for confirmation</h1>
            <p className="status__text">Your payment may still be processing. Check again in a minute, or send us your reference and we’ll look into it.</p>
            <div className="status__actions">
              <Button onClick={() => setRunId((n) => n + 1)}>Check again</Button>
              <Button variant="secondary" href={site.whatsappUrl(`Hi, I paid but haven't received confirmation. Reference: ${identifier}`)} target="_blank" rel="noreferrer">Message support</Button>
            </div>
          </>
        )}

        {state === 'done' && receipt && (
          <>
            <div className="status__icon status__icon--ok"><LuCircleCheck size={34} /></div>
            <h1 className="status__title">Payment confirmed</h1>
            <p className="status__text">Your bundle is on its way to {receipt.phone || 'the recipient'}.</p>

            <Ticket
              className="status__ticket"
              top={
                <>
                  <div className="receipt__ref">
                    <span>Reference</span>
                    <code>{identifier}</code>
                    <CopyButton text={identifier} />
                  </div>
                  <dl className="summary">
                    <div><dt>Phone</dt><dd>{receipt.phone || 'N/A'}</dd></div>
                    <div><dt>Network</dt><dd>{receipt.network ? <NetworkChip network={receipt.network} /> : 'N/A'}</dd></div>
                    <div><dt>Bundle</dt><dd>{receipt.bundle || 'N/A'}</dd></div>
                    <div><dt>Amount</dt><dd>{receipt.amount}</dd></div>
                  </dl>
                </>
              }
              bottom={
                <ol className="timeline">
                  <li className="timeline__item is-done"><LuCircleCheck aria-hidden="true" /><div><strong>Payment confirmed</strong><span>Paystack verified your payment.</span></div></li>
                  <li className="timeline__item is-active"><LuRadioTower aria-hidden="true" /><div><strong>Sending to the network</strong><span>The bundle usually arrives within 2 minutes.</span></div></li>
                </ol>
              }
            />

            <div className="status__actions">
              {ref ? (
                <Button to={`/track?ref=${encodeURIComponent(ref)}`}>Track this order</Button>
              ) : (
                <Button to="/account/orders">View my orders</Button>
              )}
              <Button to="/data" variant="secondary">Buy more data</Button>
            </div>
            <p className="status__fine">
              Not received after 30 minutes? <Link to="/support">Contact support</Link> with your reference.
            </p>
          </>
        )}
      </div>
    </div>
  );
}
