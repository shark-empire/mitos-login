import { useEffect, useRef, useState } from 'react';
import { LuCircleAlert, LuCircleCheck, LuMessageCircle } from 'react-icons/lu';
import { Button, Callout, CopyButton, NetworkChip, Spinner, TextField, Ticket } from '../ui';
import { useToast } from '../../context/Toast';
import { useOwnerLookup } from '../../hooks/useOwnerLookup';
import { config } from '../../config/env';
import { money } from '../../lib/format';
import { storeRef } from '../../lib/ids';
import { shopPath, whatsappLink } from '../../lib/links';
import { shopOrders } from '../../lib/storage';
import { digitsOnly, validatePhone, cleanPhone } from '../../lib/phone';
import { loadPaystack, openPaystack } from '../../services/paystack';
import { createOrder, toDbNetwork } from '../../services/stores';

/**
 * Guest checkout for one store bundle: recipient number, Paystack, then the order is recorded for the store to deliver.
 * `bundle.store` must be set (it carries the store's name and WhatsApp number).
 */
export default function StoreCheckout({ bundle }) {
  const toast = useToast();
  const [phone, setPhone] = useState('');
  const [touched, setTouched] = useState(false);
  const [paying, setPaying] = useState(false);
  const [receipt, setReceipt] = useState(null);
  const phoneRef = useRef(null);
  const store = bundle.store;

  useEffect(() => {
    loadPaystack().catch(() => {}); // warm up so the popup opens at once
  }, []);

  // Choosing a different bundle starts a fresh order.
  useEffect(() => {
    setReceipt(null);
    setPaying(false);
  }, [bundle.id]);

  const phoneCheck = validatePhone(phone);
  const owner = useOwnerLookup(digitsOnly(phone), bundle.network, phoneCheck.ok);

  const record = async (reference, recipient) => {
    let saved = true;
    try {
      await createOrder({ bundle, phone: recipient, reference });
    } catch {
      saved = false;
    }
    // Remember it on this device so the customer can find it again on the store's tracking page.
    shopOrders.add(store?.slug, { ref: reference, size: bundle.size, network: bundle.network, at: Date.now() });
    setReceipt({ reference, phone: recipient, saved, busy: false });
    setPaying(false);
  };

  const pay = async () => {
    setTouched(true);
    if (!phoneCheck.ok) {
      phoneRef.current?.focus();
      return;
    }
    const recipient = digitsOnly(phone);
    const reference = storeRef();
    const network = toDbNetwork(bundle.network);
    setPaying(true);
    try {
      // The metadata is what the payment automation reads, so keep its shape.
      await openPaystack({
        key: config.stores.paystackKey,
        email: `${recipient}@${config.stores.payerDomain}`,
        amount: bundle.price,
        reference,
        metadata: {
          type: 'bundle_purchase',
          phone: recipient,
          network,
          bundle_size: bundle.size,
          bundle_id: bundle.id,
          store_id: bundle.storeId,
          custom_fields: [
            { display_name: 'Phone', variable_name: 'phone', value: recipient },
            { display_name: 'Network', variable_name: 'network', value: network },
            { display_name: 'Bundle', variable_name: 'bundle_size', value: bundle.size },
            { display_name: 'Store', variable_name: 'store', value: store?.name || '' },
          ],
        },
        onSuccess: (res) => {
          // Hide the form straight away so the payment cannot be started twice while the order is saved.
          setReceipt({ reference: res.reference || reference, phone: recipient, saved: true, busy: true });
          record(res.reference || reference, recipient);
        },
        onClose: () => setPaying(false),
      });
    } catch (e) {
      toast.err(e.message || 'Could not open the payment window.');
      setPaying(false);
    }
  };

  if (receipt) {
    const chat = whatsappLink(store?.whatsapp, `Hi, I paid for ${bundle.size} ${bundle.network} for ${receipt.phone}. Reference: ${receipt.reference}`);
    return (
      <section className="panel receipt" aria-live="polite">
        {receipt.busy ? (
          <>
            <h2 className="receipt__title"><Spinner size={20} /> Saving your order</h2>
            <p className="receipt__text">Your payment went through. Please wait a moment.</p>
          </>
        ) : (
          <>
            <h2 className="receipt__title"><LuCircleCheck aria-hidden="true" /> Payment received</h2>
            <p className="receipt__text">
              {receipt.saved
                ? <>{store?.name || 'The store'} will deliver <strong>{bundle.size} {bundle.network}</strong> to <strong>{receipt.phone}</strong>.</>
                : <>We could not record the order for {store?.name || 'the store'}. Your money is safe: send the reference below to the store so they can deliver it.</>}
            </p>
            <div className="receipt__ref"><code>{receipt.reference}</code><CopyButton text={receipt.reference} /></div>
            {!receipt.saved && <Callout tone="warn" icon={LuCircleAlert}>Keep this reference. It is your proof of payment.</Callout>}
            <p className="summary__fine">If the bundle has not arrived in 30 minutes, message the store with your reference.</p>
            <div className="status__actions status__actions--left">
              {store?.slug && <Button to={`${shopPath(store.slug)}/track?ref=${encodeURIComponent(receipt.reference)}`}>Track this order</Button>}
              {chat && <Button href={chat} target="_blank" rel="noreferrer" variant="secondary"><LuMessageCircle aria-hidden="true" /> Message the store</Button>}
              <Button variant="ghost" onClick={() => { setReceipt(null); setPhone(''); setTouched(false); }}>Buy another</Button>
            </div>
          </>
        )}
      </section>
    );
  }

  return (
    <Ticket
      top={
        <>
          <h2 className="summary__title">Your order</h2>
          <dl className="summary">
            <div><dt>Store</dt><dd>{store?.name || '–'}</dd></div>
            <div><dt>Network</dt><dd><NetworkChip network={bundle.network} /></dd></div>
            <div><dt>Bundle</dt><dd>{bundle.size}</dd></div>
            <div><dt>Valid for</dt><dd>{bundle.validity}</dd></div>
          </dl>
        </>
      }
      bottom={
        <>
          <TextField
            label="Phone number that receives the data"
            type="tel"
            inputMode="numeric"
            autoComplete="tel-national"
            maxLength={16}
            placeholder="0541234567"
            prefix="+233"
            ref={phoneRef}
            value={phone}
            onChange={(e) => setPhone(cleanPhone(e.target.value))}
            onBlur={() => setTouched(true)}
            error={touched && phone && !phoneCheck.ok ? phoneCheck.msg : undefined}
            hint="Double-check the number. Data sent to the wrong number cannot be recalled."
          />
          <div className="owner" aria-live="polite">
            {owner.state === 'loading' && <span className="owner__line"><Spinner size={14} /> Checking the registered owner</span>}
            {owner.state === 'ok' && <span className="owner__line owner__line--ok"><LuCircleCheck aria-hidden="true" /> Registered to <strong>{owner.name}</strong></span>}
            {owner.state === 'unresolved' && <span className="owner__line owner__line--warn"><LuCircleAlert aria-hidden="true" /> We couldn’t find the owner’s name. Make sure the number is right.</span>}
          </div>
          <div className="summary__total">
            <span>Total</span>
            <strong>{money(bundle.price)}</strong>
          </div>
          <Button block size="lg" onClick={pay} loading={paying} data-magnetic>
            Pay {money(bundle.price)}
          </Button>
          <p className="summary__fine">Mobile Money or card, processed by Paystack. We never see your PIN or card details.</p>
        </>
      }
    />
  );
}
