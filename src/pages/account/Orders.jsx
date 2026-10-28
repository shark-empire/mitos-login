import { useEffect, useState } from 'react';
import { LuPackage } from 'react-icons/lu';
import { Button, Empty, NetworkChip, PageHeader, StatusBadge } from '../../components/ui';
import { useAuth } from '../../context/Auth';
import { usePageTitle } from '../../hooks/usePageTitle';
import { accountsApi } from '../../services/rest';
import { dateTime, money } from '../../lib/format';
import { normalizeNetwork } from '../../config/site';

export function OrdersTable({ orders }) {
  return (
    <div className="tablewrap">
      <table className="datatable datatable--stack">
        <thead>
          <tr>
            <th scope="col">Order</th>
            <th scope="col">Bundle</th>
            <th scope="col">Amount</th>
            <th scope="col">Paid with</th>
            <th scope="col">Status</th>
            <th scope="col">Date</th>
          </tr>
        </thead>
        <tbody>
          {orders.map((o) => (
            <tr key={o.order_id || o.id}>
              <th scope="row" data-label="Order"><code>{o.order_id || o.id}</code></th>
              <td data-label="Bundle">
                <span className="cell-group">
                  {o.size || o.bundle_id || '–'}
                  {o.network && <NetworkChip network={normalizeNetwork(o.network)} />}
                </span>
              </td>
              <td data-label="Amount">{money(o.amount)}</td>
              <td data-label="Paid with" className="cap">{o.payment_method || '–'}</td>
              <td data-label="Status"><StatusBadge status={o.status} /></td>
              <td data-label="Date">{dateTime(o.created_at)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export default function Orders() {
  usePageTitle('My orders');
  const { user, authed } = useAuth();
  const [orders, setOrders] = useState(null);
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    let off = false;
    authed((t) => accountsApi.select('orders', `?user_id=eq.${user.id}&order=created_at.desc`, t))
      .then((rows) => !off && setOrders(rows || []))
      .catch(() => {
        if (!off) {
          setFailed(true);
          setOrders([]);
        }
      });
    return () => {
      off = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <>
      <PageHeader title="My orders" lead="Orders placed while signed in. Paid as a guest? Use Track order with your reference." actions={<Button to="/track" variant="secondary" size="sm">Track order</Button>} />
      <section className="panel">
        {orders === null && <div className="skeleton-block" aria-busy="true" />}
        {orders && failed && <Empty icon={LuPackage} title="We couldn’t load your orders">Check your connection and reload the page.</Empty>}
        {orders && !failed && orders.length === 0 && (
          <Empty icon={LuPackage} title="No orders yet" action={<Button to="/data" size="sm">Buy data</Button>}>Your orders will appear here.</Empty>
        )}
        {orders && orders.length > 0 && <OrdersTable orders={orders} />}
      </section>
    </>
  );
}
