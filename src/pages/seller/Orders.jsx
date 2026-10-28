import { useCallback, useEffect, useState } from 'react';
import { LuRefreshCw, LuShoppingBag } from 'react-icons/lu';
import { Button, Empty, PageHeader, StatusBadge } from '../../components/ui';
import { useMyStore } from '../../components/layout/SellerLayout';
import { useSeller } from '../../context/Seller';
import { usePageTitle } from '../../hooks/usePageTitle';
import { dateTime, money } from '../../lib/format';
import { fetchOrdersByStore } from '../../services/stores';

export function OrdersTable({ orders }) {
  return (
    <div className="tablewrap">
      <table className="datatable datatable--stack">
        <thead>
          <tr>
            <th scope="col">Reference</th>
            <th scope="col">Customer number</th>
            <th scope="col">Amount</th>
            <th scope="col">Status</th>
            <th scope="col">Date</th>
          </tr>
        </thead>
        <tbody>
          {orders.map((o) => (
            <tr key={o.id}>
              <th scope="row" data-label="Reference"><code>{o.reference || String(o.id).slice(0, 8)}</code></th>
              <td data-label="Customer number">{o.phone || '–'}</td>
              <td data-label="Amount">{money(o.amount)}</td>
              <td data-label="Status"><StatusBadge status={o.status} /></td>
              <td data-label="Date">{dateTime(o.createdAt)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export default function Orders() {
  usePageTitle('Store orders');
  const { store } = useMyStore();
  const { authed } = useSeller();
  const [orders, setOrders] = useState(null);
  const [failed, setFailed] = useState(false);
  const [busy, setBusy] = useState(false);

  const load = useCallback(() => {
    setBusy(true);
    setFailed(false);
    return authed((t) => fetchOrdersByStore(store.id, t))
      .then(setOrders)
      .catch(() => {
        setFailed(true);
        setOrders((o) => o || []);
      })
      .finally(() => setBusy(false));
  }, [store.id, authed]);

  useEffect(() => {
    load();
  }, [load]);

  return (
    <>
      <PageHeader
        title="Orders"
        lead="Paid orders from customers. “Processing” means the payment is being confirmed or the data is on its way."
        actions={<Button variant="secondary" size="sm" onClick={load} loading={busy}><LuRefreshCw aria-hidden="true" /> Refresh</Button>}
      />
      <section className="panel">
        {orders === null && <div className="skeleton-block" aria-busy="true" />}
        {orders && failed && <Empty icon={LuShoppingBag} title="We couldn’t load your orders">Check your connection and press Refresh.</Empty>}
        {orders && !failed && orders.length === 0 && <Empty icon={LuShoppingBag} title="No orders yet">Orders appear here as soon as a customer pays.</Empty>}
        {orders && orders.length > 0 && <OrdersTable orders={orders} />}
      </section>
    </>
  );
}
