import { useEffect, useMemo, useState } from 'react';
import { LuPackage, LuShoppingBag } from 'react-icons/lu';
import { Button, CopyButton, Empty, PageHeader } from '../../components/ui';
import { useMyStore } from '../../components/layout/SellerLayout';
import { OrdersTable } from './Orders';
import { useSeller } from '../../context/Seller';
import { usePageTitle } from '../../hooks/usePageTitle';
import { money } from '../../lib/format';
import { shopPath, storeUrl } from '../../lib/links';
import { fetchOrdersByStore, fetchStoreStats } from '../../services/stores';

const DONE = ['completed', 'delivered', 'success'];
const WAITING = ['pending', 'processing'];
const isDone = (o) => DONE.includes(String(o.status).toLowerCase());

/** Totals worked out from the orders we have, for when the database function is not there. */
const compute = (orders) => {
  const today = new Date().toDateString();
  return {
    sales: orders.filter(isDone).reduce((n, o) => n + o.amount, 0),
    today: orders.filter((o) => new Date(o.createdAt).toDateString() === today).length,
    orders: orders.length,
    customers: new Set(orders.map((o) => o.phone).filter(Boolean)).size,
  };
};

export default function Overview() {
  usePageTitle('Seller dashboard');
  const { store } = useMyStore();
  const { authed } = useSeller();
  const [orders, setOrders] = useState(null);
  const [stats, setStats] = useState(null);

  useEffect(() => {
    let off = false;
    authed((t) => fetchOrdersByStore(store.id, t))
      .then((rows) => !off && setOrders(rows))
      .catch(() => !off && setOrders([]));
    authed((t) => fetchStoreStats(store.id, t)).then((s) => !off && setStats(s)).catch(() => {});
    return () => {
      off = true;
    };
  }, [store.id, authed]);

  const shown = useMemo(() => stats || (orders ? compute(orders) : null), [stats, orders]);
  const waiting = (orders || []).filter((o) => WAITING.includes(String(o.status).toLowerCase())).length;
  const link = storeUrl(store.slug);

  return (
    <>
      <PageHeader
        title={store.name}
        lead="How your store is doing."
        actions={<Button to="/seller/bundles" size="sm"><LuPackage aria-hidden="true" /> Add bundles</Button>}
      />

      <div className="stats">
        <div className="stat stat--primary">
          <span className="stat__label">Sales</span>
          <strong className="stat__value">{shown ? money(shown.sales) : '–'}</strong>
        </div>
        <div className="stat"><span className="stat__label">Orders</span><strong className="stat__value">{shown ? shown.orders : '–'}</strong></div>
        <div className="stat"><span className="stat__label">In progress</span><strong className="stat__value">{orders ? waiting : '–'}</strong></div>
        <div className="stat"><span className="stat__label">Customers</span><strong className="stat__value">{shown ? shown.customers : '–'}</strong></div>
      </div>

      <section className="panel">
        <div className="panel__head"><h2 className="panel__title">Your store link</h2></div>
        <p className="summary__fine">Share this with customers. It opens only your store, its checkout and an order tracker. Customers never see the rest of the site.</p>
        <div className="receipt__ref"><code>{link}</code><CopyButton text={link} label="Copy link" /></div>
      </section>

      <section className="panel">
        <div className="panel__head">
          <h2 className="panel__title">Recent orders</h2>
          <Button to="/seller/orders" variant="ghost" size="sm">View all</Button>
        </div>
        {orders === null ? (
          <div className="skeleton-block" aria-busy="true" />
        ) : orders.length === 0 ? (
          <Empty icon={LuShoppingBag} title="No orders yet" action={<Button to={shopPath(store.slug)} size="sm">View my store</Button>}>
            Orders appear here as soon as a customer pays.
          </Empty>
        ) : (
          <OrdersTable orders={orders.slice(0, 5)} />
        )}
      </section>
    </>
  );
}
