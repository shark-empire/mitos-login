import { createContext, useCallback, useContext, useEffect, useState } from 'react';
import { LuCircleAlert, LuLayoutDashboard, LuLogOut, LuPackage, LuShoppingBag, LuStore } from 'react-icons/lu';
import { NavLink, useNavigate } from '../../router';
import { Button, Empty } from '../ui';
import { StoreAvatar } from '../stores/StoreBits';
import { useSeller } from '../../context/Seller';
import { useToast } from '../../context/Toast';
import { fetchStoreByOwner } from '../../services/stores';
import { shopPath } from '../../lib/links';

const StoreContext = createContext(null);
/** The signed-in seller's store, for the pages inside the seller area. */
export const useMyStore = () => useContext(StoreContext);

const ITEMS = [
  { to: '/seller', label: 'Overview', icon: LuLayoutDashboard, end: true },
  { to: '/seller/orders', label: 'Orders', icon: LuShoppingBag },
  { to: '/seller/bundles', label: 'Bundles', icon: LuPackage },
];

export default function SellerLayout({ children }) {
  const { seller, signOut, authed } = useSeller();
  const toast = useToast();
  const navigate = useNavigate();
  const [store, setStore] = useState(undefined); // undefined: loading, null: no store yet
  const [failed, setFailed] = useState(false);

  const load = useCallback(() => {
    setFailed(false);
    setStore(undefined);
    authed((t) => fetchStoreByOwner(seller.id, t))
      .then((s) => setStore(s || null))
      .catch(() => {
        setFailed(true);
        setStore(null);
      });
  }, [authed, seller.id]);

  useEffect(load, [load]);

  const out = () => {
    signOut();
    toast.info('You have been signed out.');
    navigate('/');
  };

  let body;
  if (store === undefined) body = <div className="skeleton-block" aria-busy="true" />;
  else if (failed) {
    body = (
      <Empty icon={LuCircleAlert} title="We couldn’t load your store" action={<Button size="sm" onClick={load}>Try again</Button>}>
        Check your connection and try again.
      </Empty>
    );
  } else if (!store) {
    body = (
      <Empty icon={LuStore} title="You haven’t opened a store yet" action={<Button to="/store-builder" size="sm">Open your store</Button>}>
        It takes a couple of minutes: a name, a colour and your first bundles.
      </Empty>
    );
  } else body = <StoreContext.Provider value={{ store, reload: load }}>{children}</StoreContext.Provider>;

  return (
    <div className="container account">
      <aside className="account__side">
        <div className="account__who">
          {store ? <StoreAvatar store={store} size="sm" /> : <span className="account__avatar" aria-hidden="true">{(seller.name || seller.email || '?').charAt(0).toUpperCase()}</span>}
          <div className="account__whotext">
            <strong>{store?.name || seller.name || 'Your store'}</strong>
            <span>{seller.email}</span>
          </div>
        </div>
        <nav className="account__nav" aria-label="Seller">
          {ITEMS.map(({ to, label, icon: Icon, end }) => (
            <NavLink key={to} to={to} end={end} className="account__link">
              <Icon aria-hidden="true" />
              {label}
            </NavLink>
          ))}
          {store && (
            <NavLink to={shopPath(store.slug)} end className="account__link">
              <LuStore aria-hidden="true" />
              View my store
            </NavLink>
          )}
          <button type="button" className="account__link account__signout" onClick={out}>
            <LuLogOut aria-hidden="true" />
            Sign out
          </button>
        </nav>
      </aside>
      <section className="account__main">{body}</section>
    </div>
  );
}
