import { useEffect, useRef, useState } from 'react';
import { Button, Callout, PageHeader, TextField } from '../../components/ui';
import { LuInfo } from 'react-icons/lu';
import { useAuth } from '../../context/Auth';
import { useToast } from '../../context/Toast';
import { usePageTitle } from '../../hooks/usePageTitle';
import { config } from '../../config/env';
import { money } from '../../lib/format';
import { walletRef } from '../../lib/ids';
import { walletTopUp } from '../../lib/fees';
import { loadPaystack, openPaystack } from '../../services/paystack';

const QUICK = [10, 20, 50, 100, 200];

export default function Fund() {
  usePageTitle('Add money');
  const { user, reloadProfile } = useAuth();
  const toast = useToast();
  const [amount, setAmount] = useState('');
  const [busy, setBusy] = useState(false);
  const [waiting, setWaiting] = useState(false);
  const startBalance = useRef(0);
  const timer = useRef(null);

  useEffect(() => {
    loadPaystack().catch(() => {});
    return () => clearInterval(timer.current);
  }, []);

  const value = parseFloat(amount);
  const valid = Number.isFinite(value) && value >= 1;
  const cost = walletTopUp(valid ? value : 0);
  const hasFee = cost.percent > 0;

  // The balance is credited by the payment automation, so check for it for a minute after paying.
  const watchBalance = () => {
    startBalance.current = user.wallet_balance;
    setWaiting(true);
    let tries = 0;
    clearInterval(timer.current);
    timer.current = setInterval(async () => {
      tries += 1;
      const profile = await reloadProfile().catch(() => null);
      if (profile && Number(profile.wallet_balance) > startBalance.current) {
        clearInterval(timer.current);
        setWaiting(false);
        toast.ok('Your wallet has been topped up.');
      } else if (tries >= 15) {
        clearInterval(timer.current);
        setWaiting(false);
        toast.info('Your top-up is still processing. It will show in your wallet shortly.');
      }
    }, 4000);
  };

  const pay = async () => {
    if (!valid) return toast.err('Enter an amount of GH₵ 1 or more.');
    setBusy(true);
    try {
      await openPaystack({
        key: config.paystack.dataKey,
        email: user.email,
        // The customer is charged the amount plus the Paystack fee. `wallet_credit` is what the wallet should receive,
        // so whatever credits the balance must use it rather than the total that Paystack reports.
        amount: cost.total,
        reference: walletRef(),
        metadata: {
          type: 'wallet_funding',
          user_id: user.id,
          wallet_credit: cost.credit,
          fee: cost.fee,
          fee_percent: cost.percent,
          custom_fields: [
            { display_name: 'Wallet credit', variable_name: 'wallet_credit', value: cost.credit },
            { display_name: 'Payment fee', variable_name: 'fee', value: cost.fee },
          ],
        },
        onSuccess: () => {
          setAmount('');
          setBusy(false);
          watchBalance();
        },
        onClose: () => setBusy(false),
      });
    } catch (e) {
      toast.err(e.message || 'Could not open the payment window.');
      setBusy(false);
    }
  };

  return (
    <>
      <PageHeader title="Add money" lead="Top up your wallet with Mobile Money or card." />
      <div className="fundgrid">
        <section className="panel">
          <p className="fund__balance"><span>Current balance</span><strong>{money(user.wallet_balance)}</strong></p>
          <TextField label="Amount (GH₵)" type="number" inputMode="decimal" min="1" step="1" placeholder="50" value={amount} onChange={(e) => setAmount(e.target.value)} />
          <div className="quick" role="group" aria-label="Quick amounts">
            {QUICK.map((q) => (
              <button key={q} type="button" className={`quick__item${value === q ? ' is-on' : ''}`} onClick={() => setAmount(String(q))}>{money(q).replace('.00', '')}</button>
            ))}
          </div>
          {valid && (
            <dl className="summary fund__breakdown" aria-live="polite">
              <div><dt>Added to your wallet</dt><dd>{money(cost.credit)}</dd></div>
              {hasFee && <div><dt>Paystack fee ({cost.percent}%)</dt><dd>{money(cost.fee)}</dd></div>}
              <div className="fund__total"><dt>You pay</dt><dd>{money(cost.total)}</dd></div>
            </dl>
          )}
          <Button block size="lg" onClick={pay} loading={busy || waiting} disabled={!valid}>
            {waiting ? 'Waiting for your balance' : valid ? `Pay ${money(cost.total)}` : 'Enter an amount'}
          </Button>
          <p className="summary__fine">
            Payments are processed by Paystack.{hasFee ? ` The ${cost.percent}% fee is Paystack’s processing charge, so it is added to what you pay.` : ''} Your balance updates once the payment is confirmed.
          </p>
        </section>
        <Callout tone="info" icon={LuInfo}>
          Money in your wallet is not refundable once it has been added. You can spend it on any data bundle.{hasFee ? ' The Paystack fee is not added to your balance.' : ''}
        </Callout>
      </div>
    </>
  );
}
