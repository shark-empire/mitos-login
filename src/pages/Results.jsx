import { useEffect, useMemo, useState } from 'react';
import { LuCircleAlert, LuDownload, LuMinus, LuPlus, LuTag, LuTicket } from 'react-icons/lu';
import { Button, Callout, CopyButton, PageHeader, Spinner, TextField, Ticket } from '../components/ui';
import { useToast } from '../context/Toast';
import { useAuth } from '../context/Auth';
import { usePageTitle } from '../hooks/usePageTitle';
import { useTint } from '../hooks/useTint';
import { config } from '../config/env';
import {
  RESULTS_BULK_PRICE,
  RESULTS_BULK_THRESHOLD,
  RESULTS_PRODUCTS,
  RESULTS_REGULAR_PRICE,
  site,
} from '../config/site';
import { money } from '../lib/format';
import { resultsRef } from '../lib/ids';
import { validatePhone, digitsOnly, cleanPhone } from '../lib/phone';
import { fetchResultsPins } from '../services/webhooks';
import { loadPaystack, openPaystack } from '../services/paystack';

const unitPriceFor = (qty) => (qty > RESULTS_BULK_THRESHOLD ? RESULTS_BULK_PRICE : RESULTS_REGULAR_PRICE);
const MAX_QTY = 500;
const isTestKey = config.paystack.resultsKey.startsWith('pk_test');

export default function Results() {
  usePageTitle('Results checkers');
  useTint('results');
  const toast = useToast();
  const { user } = useAuth();

  const [productId, setProductId] = useState(RESULTS_PRODUCTS[0].id);
  const [quantity, setQuantity] = useState(1);
  const [email, setEmail] = useState('');
  const [phone, setPhone] = useState('');
  const [touched, setTouched] = useState(false);
  const [phase, setPhase] = useState('form'); // form | verifying | done | error
  const [pins, setPins] = useState([]);
  const [lastRef, setLastRef] = useState('');
  const [snapshot, setSnapshot] = useState(null); // what was bought, kept for the voucher screen

  useEffect(() => {
    loadPaystack().catch(() => {});
  }, []);
  useEffect(() => {
    if (user) {
      setEmail((e) => e || user.email || '');
      setPhone((p) => p || user.phone || '');
    }
  }, [user]);

  const product = RESULTS_PRODUCTS.find((p) => p.id === productId);
  const unit = unitPriceFor(quantity);
  const bulk = quantity > RESULTS_BULK_THRESHOLD;
  const total = unit * quantity;
  const savings = bulk ? (RESULTS_REGULAR_PRICE - RESULTS_BULK_PRICE) * quantity : 0;

  const emailOk = /^\S+@\S+\.\S+$/.test(email.trim());
  const phoneCheck = validatePhone(phone);

  const setQty = (n) => setQuantity(Math.min(MAX_QTY, Math.max(1, Number.isFinite(n) ? n : 1)));

  const verify = async (reference, snap) => {
    setPhase('verifying');
    try {
      const data = await fetchResultsPins({ reference, productId: snap.productId, quantity: snap.quantity });
      const list = Array.isArray(data.pins) ? data.pins : [];
      if (!list.length) throw new Error('No vouchers returned');
      setPins(list);
      setPhase('done');
    } catch {
      setPhase('error');
    }
  };

  const pay = async () => {
    setTouched(true);
    if (!emailOk || !phoneCheck.ok) {
      toast.err('Enter your email and a valid phone number first.');
      return;
    }
    const reference = resultsRef();
    const snap = { productId: product.id, name: product.full, quantity, email: email.trim() };
    try {
      await openPaystack({
        key: config.paystack.resultsKey,
        email: snap.email,
        amount: total,
        reference,
        metadata: {
          phone: digitsOnly(phone),
          product_id: product.id,
          product_name: product.full,
          quantity,
          unit_price: unit,
          bulk_discount_applied: bulk,
        },
        onSuccess: (res) => {
          setLastRef(res.reference);
          setSnapshot(snap);
          verify(res.reference, snap);
        },
        onClose: () => {},
      });
    } catch (e) {
      toast.err(e.message || 'Could not open the payment window.');
    }
  };

  const reset = () => {
    setPhase('form');
    setPins([]);
    setQuantity(1);
  };

  const download = () => {
    const lines = pins.map((p, i) => `Voucher ${i + 1}\nPIN: ${p.pin}\nSerial number: ${p.serial}\n`);
    const text = `${snapshot?.name || 'Results checker'}\nReference: ${lastRef}\n\n${lines.join('\n')}`;
    const url = URL.createObjectURL(new Blob([text], { type: 'text/plain' }));
    const a = document.createElement('a');
    a.href = url;
    a.download = `results-checker-${lastRef}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const allText = useMemo(
    () => pins.map((p, i) => `Voucher ${i + 1}: PIN ${p.pin}, Serial ${p.serial}`).join('\n'),
    [pins],
  );

  if (phase === 'verifying') {
    return (
      <div className="container page page--narrow">
        <div className="status">
          <div className="status__icon status__icon--busy"><Spinner size={34} /></div>
          <h1 className="status__title">Getting your {quantity > 1 ? 'vouchers' : 'voucher'}</h1>
          <p className="status__text">Payment received. Please keep this page open while we generate your PIN{quantity > 1 ? 's' : ''}.</p>
        </div>
      </div>
    );
  }

  if (phase === 'error') {
    return (
      <div className="container page page--narrow">
        <div className="status">
          <div className="status__icon status__icon--warn"><LuCircleAlert size={34} /></div>
          <h1 className="status__title">Payment received, vouchers not shown yet</h1>
          <p className="status__text">
            Your payment went through, but we couldn’t fetch your PINs just now. Don’t pay again. Send us this reference and we’ll
            sort it out.
          </p>
          <p className="receipt__ref receipt__ref--center"><span>Reference</span><code>{lastRef}</code><CopyButton text={lastRef} /></p>
          <div className="status__actions">
            <Button onClick={() => verify(lastRef, snapshot)}>Try again</Button>
            <Button variant="secondary" href={site.whatsappUrl(`Hi, I paid for a results checker but did not get my PIN. Reference: ${lastRef}`)} target="_blank" rel="noreferrer">Message support</Button>
          </div>
        </div>
      </div>
    );
  }

  if (phase === 'done') {
    return (
      <div className="container page">
        <PageHeader
          title={pins.length > 1 ? 'Your vouchers' : 'Your voucher'}
          lead={`${snapshot?.name}. Keep these safe: anyone with the PIN and serial number can use them. A copy was also sent to ${snapshot?.email}.`}
          actions={
            <>
              <CopyButton text={allText} label="Copy all" />
              <Button variant="secondary" size="sm" onClick={download}><LuDownload aria-hidden="true" /> Download</Button>
            </>
          }
        />
        <ul className="vouchers">
          {pins.map((p, i) => (
            <li key={`${p.serial}-${i}`}>
              <Ticket
                top={<div className="voucher__head"><LuTicket aria-hidden="true" /> {snapshot?.name}<span className="voucher__no">Voucher {i + 1}</span></div>}
                bottom={
                  <dl className="voucher__codes">
                    <div><dt>PIN</dt><dd><code>{p.pin}</code><CopyButton text={String(p.pin)} /></dd></div>
                    <div><dt>Serial number</dt><dd><code>{p.serial}</code><CopyButton text={String(p.serial)} /></dd></div>
                  </dl>
                }
              />
            </li>
          ))}
        </ul>
        <div className="status__actions status__actions--left">
          <Button onClick={reset}>Buy more checkers</Button>
          <p className="status__fine">Reference {lastRef}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="container page">
      <PageHeader title="Results checkers" lead="Buy a WASSCE, NOVDEC or BECE results checker. Your PIN and serial number appear right after payment." />

      {isTestKey && (
        <Callout tone="info" icon={LuTag}>
          <strong>Test mode.</strong> Payments on this page use Paystack’s test keys, so no real money is charged.
        </Callout>
      )}

      <div className="buy">
        <div className="buy__steps">
          <section className="step" aria-labelledby="r-product">
            <h2 id="r-product" className="step__title"><span className="step__n">1</span> Exam</h2>
            <div className="products" role="radiogroup" aria-label="Exam">
              {RESULTS_PRODUCTS.map((p) => (
                <button
                  key={p.id}
                  type="button"
                  role="radio"
                  aria-checked={productId === p.id}
                  className={`product${productId === p.id ? ' is-on' : ''}`}
                  onClick={() => setProductId(p.id)}
                >
                  <span className="product__name">{p.name}</span>
                  <span className="product__desc">{p.desc}</span>
                  <span className="product__price">{money(unit)}</span>
                </button>
              ))}
            </div>
          </section>

          <section className="step" aria-labelledby="r-qty">
            <h2 id="r-qty" className="step__title"><span className="step__n">2</span> How many?</h2>
            <div className="qty">
              <div className="stepper" role="group" aria-label="Quantity">
                <button type="button" onClick={() => setQty(quantity - 1)} aria-label="One fewer" disabled={quantity <= 1}><LuMinus aria-hidden="true" /></button>
                <input
                  aria-label="Quantity"
                  inputMode="numeric"
                  value={quantity}
                  onChange={(e) => setQty(parseInt(e.target.value.replace(/\D/g, ''), 10))}
                />
                <button type="button" onClick={() => setQty(quantity + 1)} aria-label="One more" disabled={quantity >= MAX_QTY}><LuPlus aria-hidden="true" /></button>
              </div>
              {bulk ? (
                <p className="qty__note qty__note--on"><LuTag aria-hidden="true" /> Bulk price: {money(RESULTS_BULK_PRICE)} each. You save {money(savings)}.</p>
              ) : (
                <p className="qty__note">Buy more than {RESULTS_BULK_THRESHOLD} and each one costs {money(RESULTS_BULK_PRICE)}.</p>
              )}
            </div>
          </section>

          <section className="step" aria-labelledby="r-you">
            <h2 id="r-you" className="step__title"><span className="step__n">3</span> Your details</h2>
            <div className="fields2">
              <TextField
                label="Email address"
                type="email"
                autoComplete="email"
                placeholder="you@example.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                error={touched && !emailOk ? 'Enter a valid email address' : undefined}
                hint="We send a copy of your vouchers here."
              />
              <TextField
                label="Phone number"
                type="tel"
                inputMode="numeric"
                autoComplete="tel-national"
                maxLength={16}
                prefix="+233"
                placeholder="0541234567"
                value={phone}
                onChange={(e) => setPhone(cleanPhone(e.target.value))}
                error={touched && !phoneCheck.ok ? phoneCheck.msg : undefined}
                hint="WhatsApp or SMS, in case we need to reach you."
              />
            </div>
          </section>
        </div>

        <aside className="buy__summary" aria-label="Order summary">
          <Ticket
            top={
              <>
                <h2 className="summary__title">Your order</h2>
                <dl className="summary">
                  <div><dt>Item</dt><dd>{product.full}</dd></div>
                  <div><dt>Price</dt><dd>{quantity} × {money(unit)}</dd></div>
                  {bulk && <div className="summary__save"><dt>Bulk saving</dt><dd>−{money(savings)}</dd></div>}
                </dl>
              </>
            }
            bottom={
              <>
                <div className="summary__total"><span>Total</span><strong>{money(total)}</strong></div>
                <Button block size="lg" onClick={pay}>Pay {money(total)}</Button>
                <p className="summary__fine">Payments are processed by Paystack. Vouchers are shown on screen after payment.</p>
              </>
            }
          />
        </aside>
      </div>
    </div>
  );
}
