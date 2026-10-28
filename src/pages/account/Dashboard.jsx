import { useEffect, useState } from 'react';
import { LuCreditCard, LuPackage, LuSmartphone } from 'react-icons/lu';
import { Button, Empty, PageHeader } from '../../components/ui';
import { useAuth } from '../../context/Auth';
import { usePageTitle } from '../../hooks/usePageTitle';
import { accountsApi } from '../../services/rest';
import { money } from '../../lib/format';
import { OrdersTable } from './Orders';

export default function Dashboard() {
  usePageTitle('Account');
  const { user, authed, reloadProfile } = useAuth();
  const [orders, setOrders] = useState(null);

  useEffect(() => {
    let off = false;
    reloadProfile().catch(() => {});
    authed((t) => accountsApi.select('orders', `?user_id=eq.${user.id}&order=created_at.desc&limit=50`, t))
      .then((rows) => !off && setOrders(rows || []))
      .catch(() => !off && setOrders([]));
    return () => {
      off = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const done = (orders || []).filter((o) => ['completed', 'delivered', 'success'].includes(String(o.status).toLowerCase())).length;
  const pending = (orders || []).filter((o) => ['processing', 'pending'].includes(String(o.status).toLowerCase())).length;

  return (
    <>
      <PageHeader
        title={`Hello${user.first_name ? `, ${user.first_name}` : ''}`}
        lead="Your balance and recent orders."
        actions={<Button to="/data"><LuSmartphone aria-hidden="true" /> Buy data</Button>}
      />

      <div className="stats">
        <div className="stat stat--primary">
          <span className="stat__label">Wallet balance</span>
          <strong className="stat__value">{money(user.wallet_balance)}</strong>
          <Button to="/account/fund" variant="onDark" size="sm"><LuCreditCard aria-hidden="true" /> Add money</Button>
        </div>
        <div className="stat"><span className="stat__label">Orders</span><strong className="stat__value">{orders ? orders.length : '–'}</strong></div>
        <div className="stat"><span className="stat__label">Delivered</span><strong className="stat__value">{orders ? done : '–'}</strong></div>
        <div className="stat"><span className="stat__label">In progress</span><strong className="stat__value">{orders ? pending : '–'}</strong></div>
      </div>

      <section className="panel">
        <div className="panel__head">
          <h2 className="panel__title">Recent orders</h2>
          <Button to="/account/orders" variant="ghost" size="sm">View all</Button>
        </div>
        {orders === null ? (
          <div className="skeleton-block" aria-busy="true" />
        ) : orders.length === 0 ? (
          <Empty icon={LuPackage} title="No orders yet" action={<Button to="/data" size="sm">Buy your first bundle</Button>}>
            Orders you place with your account show up here.
          </Empty>
        ) : (
          <OrdersTable orders={orders.slice(0, 5)} />
        )}
      </section>
    </>
  );
}
