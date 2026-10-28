import { useEffect, useState } from 'react';
import { LuCreditCard, LuSmartphone, LuWallet } from 'react-icons/lu';
import { Badge, Button, Empty, PageHeader } from '../../components/ui';
import { useAuth } from '../../context/Auth';
import { usePageTitle } from '../../hooks/usePageTitle';
import { accountsApi } from '../../services/rest';
import { dateTime, money } from '../../lib/format';

export default function Wallet() {
  usePageTitle('Wallet');
  const { user, authed, reloadProfile } = useAuth();
  const [txns, setTxns] = useState(null);

  useEffect(() => {
    let off = false;
    reloadProfile().catch(() => {});
    authed((t) => accountsApi.select('wallet_transactions', `?user_id=eq.${user.id}&order=created_at.desc`, t))
      .then((rows) => !off && setTxns(rows || []))
      .catch(() => !off && setTxns([]));
    return () => {
      off = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <>
      <PageHeader title="Wallet" lead="Money you add here can be used to pay for bundles in one tap." />
      <section className="walletcard">
        <span className="walletcard__label">Available balance</span>
        <strong className="walletcard__value">{money(user.wallet_balance)}</strong>
        <div className="walletcard__actions">
          <Button to="/account/fund" variant="gold"><LuCreditCard aria-hidden="true" /> Add money</Button>
          <Button to="/data" variant="onDark"><LuSmartphone aria-hidden="true" /> Buy data</Button>
        </div>
      </section>

      <section className="panel">
        <h2 className="panel__title">Transactions</h2>
        {txns === null && <div className="skeleton-block" aria-busy="true" />}
        {txns && txns.length === 0 && <Empty icon={LuWallet} title="No transactions yet" action={<Button to="/account/fund" size="sm">Add money</Button>}>Top-ups and wallet purchases will be listed here.</Empty>}
        {txns && txns.length > 0 && (
          <div className="tablewrap">
            <table className="datatable datatable--stack">
              <thead><tr><th scope="col">Description</th><th scope="col">Type</th><th scope="col">Amount</th><th scope="col">Date</th></tr></thead>
              <tbody>
                {txns.map((t, i) => {
                  const credit = t.type === 'credit';
                  return (
                    <tr key={t.id ?? i}>
                      <th scope="row" data-label="Description">{t.description || (credit ? 'Top-up' : 'Purchase')}</th>
                      <td data-label="Type"><Badge tone={credit ? 'ok' : 'muted'}>{credit ? 'Money in' : 'Money out'}</Badge></td>
                      <td data-label="Amount" className={credit ? 'amount amount--in' : 'amount'}>{credit ? '+' : '−'}{money(Math.abs(t.amount))}</td>
                      <td data-label="Date">{dateTime(t.created_at)}</td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </>
  );
}
