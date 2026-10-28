import { useEffect, useMemo, useRef, useState } from 'react';
import { LuCircleAlert, LuCircleCheck, LuRefreshCw, LuSmartphone } from 'react-icons/lu';
import { Button, Callout, Empty, NetworkChip, PageHeader, Segmented, Spinner, TextField, Ticket } from '../components/ui';
import { useAuth } from '../context/Auth';
import { useToast } from '../context/Toast';
import { useBundles } from '../hooks/useBundles';
import { usePageTitle } from '../hooks/usePageTitle';
import { useTint } from '../hooks/useTint';
import { useLocation, useNavigate } from '../router';
import { config } from '../config/env';
import { NETWORKS, networkById } from '../config/site';
import { money, splitSize } from '../lib/format';
import { dataRef, uuid } from '../lib/ids';
import { validatePhone, digitsOnly, cleanPhone } from '../lib/phone';
import { recentRefs } from '../lib/storage';
import { lookupOwner, purchaseWithWallet, walletPaymentsEnabled } from '../services/webhooks';
import { loadPaystack, openPaystack } from '../services/paystack';

export default function BuyData() {
  usePageTitle('Buy data');
  const { search } = useLocation();
  const navigate = useNavigate();
  const toast = useToast();
  const { user, authed } = useAuth();
  const { bundles, status, reload } = useBundles();

  const [network, setNetwork] = useState(search.get('network') || null);
  const [bundleId, setBundleId] = useState(search.get('bundle') || null);
  useTint(NETWORKS.find((n) => n.id === network)?.meta);
  const [phone, setPhone] = useState('');
  const [touched, setTouched] = useState(false);
  const [payWith, setPayWith] = useState('paystack');
  const [paying, setPaying] = useState(false);
  const [owner, setOwner] = useState({ state: 'idle' });
  const phoneRef = useRef(null);

  useEffect(() => {
    loadPaystack().catch(() => {}); // warm up so the popup opens instantly
  }, []);

  // Default the recipient to the signed-in user's own number.
  useEffect(() => {
    if (user?.phone && !phone) setPhone(user.phone);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [user]);

  const networkBundles = useMemo(() => bundles.filter((b) => b.network === network), [bundles, network]);
  const bundle = useMemo(() => bundles.find((b) => String(b.id) === String(bundleId)), [bundles, bundleId]);

  // A bundle from a deep link also sets the network.
  useEffect(() => {
    if (bundle && bundle.network !== network) setNetwork(bundle.network);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [bundle]);

  const phoneCheck = validatePhone(phone);
  const walletOk = !!user && walletPaymentsEnabled && bundle && user.wallet_balance >= bundle.price;
  useEffect(() => {
    if (payWith === 'wallet' && !walletOk) setPayWith('paystack');
  }, [payWith, walletOk]);

  // Ask who owns the number once it is complete (debounced, cancelled when the input changes).
  useEffect(() => {
    if (!phoneCheck.ok || !network) {
      setOwner({ state: 'idle' });
      return undefined;
    }
    const ctrl = new AbortController();
    setOwner({ state: 'loading' });
    const t = setTimeout(() => {
      lookupOwner({ phone: digitsOnly(phone), network, signal: ctrl.signal })
        .then((r) => setOwner(r.status === 'success' && r.name ? { state: 'ok', name: r.name } : { state: 'unresolved' }))
        .catch((e) => {
          if (e.name !== 'AbortError') setOwner({ state: 'error' });
        });
    }, 350);
    return () => {
      clearTimeout(t);
      ctrl.abort();
    };
  }, [phone, network, phoneCheck.ok]);

  const pickNetwork = (id) => {
    if (id === network) return;
    setNetwork(id);
    if (bundle && bundle.network !== id) setBundleId(null);
  };

  const pay = async () => {
    setTouched(true);
    if (!network) return toast.err('Choose a network first.');
    if (!bundle) return toast.err('Choose a bundle.');
    if (!phoneCheck.ok) {
      phoneRef.current?.focus();
      return undefined;
    }
    const recipient = digitsOnly(phone);
    setPaying(true);

    if (payWith === 'wallet') {
      try {
        const res = await authed((token) =>
          purchaseWithWallet({ token, userId: user.id, bundleId: bundle.id, recipientPhone: recipient }),
        );
        navigate(`/data/success?order=${encodeURIComponent(res.order_id || '')}`);
      } catch (e) {
        toast.err(e.message || 'The wallet payment failed.');
        setPaying(false);
      }
      return undefined;
    }

    // Guest-style Paystack checkout. The metadata below is read by the payment automation, so keep its shape.
    const meta = networkById(network)?.meta || String(network).toLowerCase();
    const volume = String(parseInt((String(bundle.size).match(/\d+/) || ['1'])[0], 10)); // sent as `volumeInMB`, as before
    const idempotencyKey = uuid();
    try {
      await openPaystack({
        key: config.paystack.dataKey,
        email: user?.email || `${recipient}@sci-fidata.com`,
        amount: bundle.price,
        reference: dataRef(),
        metadata: {
          phone: recipient,
          network: meta,
          volumeInMB: volume,
          bundle_size: bundle.size,
          uuid: idempotencyKey,
          ...(user ? { user_id: user.id } : {}),
          custom_fields: [
            { display_name: 'Phone Number', variable_name: 'phone', value: recipient },
            { display_name: 'Network', variable_name: 'network', value: meta },
            { display_name: 'Volume MB', variable_name: 'volume_in_gb', value: volume },
            { display_name: 'Bundle ID', variable_name: 'bundle_id', value: bundle.id },
            { display_name: 'X Idempotency Key', variable_name: 'x_idempotency_key', value: idempotencyKey },
          ],
        },
        onSuccess: (res) => {
          recentRefs.add(res.reference);
          navigate(`/data/success?ref=${encodeURIComponent(res.reference)}`);
        },
        onClose: () => setPaying(false),
      });
    } catch (e) {
      toast.err(e.message || 'Could not open the payment window.');
      setPaying(false);
    }
    return undefined;
  };

  return (
    <div className="container page">
      <PageHeader title="Buy data" lead="Choose a network and bundle, then enter the number that should receive it." />

      <div className="buy">
        <div className="buy__steps">
          <section className="step" aria-labelledby="step-network">
            <h2 id="step-network" className="step__title"><span className="step__n">1</span> Network</h2>
            <div className="nettiles" role="group" aria-label="Network">
              {NETWORKS.map((n) => (
                <button
                  key={n.id}
                  type="button"
                  className={`nettile nettile--${n.meta}${network === n.id ? ' is-on' : ''}`}
                  aria-pressed={network === n.id}
                  onClick={() => pickNetwork(n.id)}
                >
                  <span className="nettile__name">{n.long}</span>
                  <span className="nettile__note">{n.delivery}</span>
                </button>
              ))}
            </div>
          </section>

          <section className="step" aria-labelledby="step-bundle">
            <h2 id="step-bundle" className="step__title"><span className="step__n">2</span> Bundle</h2>
            {status === 'loading' && (
              <div className="bundlegrid" aria-busy="true">
                {Array.from({ length: 6 }).map((_, i) => <div key={i} className="bundle bundle--skeleton" />)}
              </div>
            )}
            {status === 'error' && (
              <Callout tone="bad" icon={LuCircleAlert}>
                <p>We couldn’t load the bundle list. Check your connection.</p>
                <Button variant="secondary" size="sm" onClick={reload}><LuRefreshCw aria-hidden="true" /> Try again</Button>
              </Callout>
            )}
            {status === 'ready' && !network && <p className="step__hint">Choose a network to see its bundles.</p>}
            {status === 'ready' && network && networkBundles.length === 0 && (
              <Empty icon={LuSmartphone} title={`No ${network} bundles right now`}>Try another network or check back soon.</Empty>
            )}
            {status === 'ready' && network && networkBundles.length > 0 && (
              <div className="bundlegrid" role="group" aria-label={`${network} bundles`}>
                {networkBundles.map((b) => {
                  const { num, unit } = splitSize(b.size);
                  return (
                    <button
                      key={b.id}
                      type="button"
                      disabled={!b.available}
                      className={`bundle${String(bundleId) === String(b.id) ? ' is-on' : ''}`}
                      aria-pressed={String(bundleId) === String(b.id)}
                      onClick={() => setBundleId(b.id)}
                    >
                      <span className="bundle__size">{num}<span className="bundle__unit">{unit}</span></span>
                      <span className="bundle__price">{b.available ? money(b.price) : 'Sold out'}</span>
                    </button>
                  );
                })}
              </div>
            )}
          </section>

          <section className="step" aria-labelledby="step-recipient">
            <h2 id="step-recipient" className="step__title"><span className="step__n">3</span> Recipient</h2>
            <TextField
              label="Phone number that receives the data"
              type="tel"
              inputMode="numeric"
              autoComplete="tel-national"
              maxLength={16}
              placeholder="0541234567"
              prefix="+233"
              value={phone}
              ref={phoneRef}
              onChange={(e) => setPhone(cleanPhone(e.target.value))}
              onBlur={() => setTouched(true)}
              error={touched && phone && !phoneCheck.ok ? phoneCheck.msg : undefined}
              hint="Double-check the number. Data sent to the wrong number cannot be recalled."
            />
            <div className="owner" aria-live="polite">
              {!network && phoneCheck.ok && <span className="owner__line">Choose a network to check who owns this number.</span>}
              {owner.state === 'loading' && <span className="owner__line"><Spinner size={14} /> Checking the registered owner</span>}
              {owner.state === 'ok' && <span className="owner__line owner__line--ok"><LuCircleCheck aria-hidden="true" /> Registered to <strong>{owner.name}</strong></span>}
              {owner.state === 'unresolved' && <span className="owner__line owner__line--warn"><LuCircleAlert aria-hidden="true" /> We couldn’t find the owner’s name. Make sure the number and network are right.</span>}
              {owner.state === 'error' && <span className="owner__line owner__line--warn"><LuCircleAlert aria-hidden="true" /> The name check is unavailable. You can still continue.</span>}
            </div>
          </section>
        </div>

        <aside className="buy__summary" aria-label="Order summary">
          <Ticket
            top={
              <>
                <h2 className="summary__title">Your order</h2>
                <dl className="summary">
                  <div><dt>Network</dt><dd>{network ? <NetworkChip network={network} /> : '–'}</dd></div>
                  <div><dt>Bundle</dt><dd>{bundle ? bundle.size : '–'}</dd></div>
                  <div><dt>Recipient</dt><dd>{phoneCheck.ok ? phone : '–'}</dd></div>
                </dl>
              </>
            }
            bottom={
              <>
                <div className="summary__total">
                  <span>Total</span>
                  <strong>{bundle ? money(bundle.price) : money(0)}</strong>
                </div>
                {user && walletPaymentsEnabled && (
                  <Segmented
                    label="Pay with"
                    value={payWith}
                    onChange={setPayWith}
                    options={[
                      { value: 'paystack', label: 'Mobile Money or card' },
                      { value: 'wallet', label: `Wallet ${money(user.wallet_balance)}`, disabled: !walletOk },
                    ]}
                  />
                )}
                <Button block size="lg" onClick={pay} loading={paying} disabled={!bundle}>
                  {paying ? 'Processing' : bundle ? `Pay ${money(bundle.price)}` : 'Choose a bundle'}
                </Button>
                <p className="summary__fine">Payments are processed by Paystack. By paying you agree to our <a href="#/terms">terms</a>.</p>
              </>
            }
          />
        </aside>
      </div>
    </div>
  );
}
