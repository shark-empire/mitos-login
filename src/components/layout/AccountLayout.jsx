import { LuCreditCard, LuLayoutDashboard, LuLogOut, LuPackage, LuSettings, LuSmartphone, LuWallet } from 'react-icons/lu';
import { NavLink } from '../../router';
import { useAuth } from '../../context/Auth';
import { useToast } from '../../context/Toast';
import { useNavigate } from '../../router';

const ITEMS = [
  { to: '/account', label: 'Overview', icon: LuLayoutDashboard, end: true },
  { to: '/account/orders', label: 'Orders', icon: LuPackage },
  { to: '/account/wallet', label: 'Wallet', icon: LuWallet },
  { to: '/account/fund', label: 'Add money', icon: LuCreditCard },
  { to: '/account/settings', label: 'Settings', icon: LuSettings },
];

export default function AccountLayout({ children }) {
  const { user, signOut } = useAuth();
  const toast = useToast();
  const navigate = useNavigate();

  const out = () => {
    signOut();
    toast.info('You have been signed out.');
    navigate('/');
  };

  return (
    <div className="container account">
      <aside className="account__side">
        <div className="account__who">
          <span className="account__avatar" aria-hidden="true">
            {(user.first_name || user.email || '?').charAt(0).toUpperCase()}
          </span>
          <div className="account__whotext">
            <strong>{[user.first_name, user.last_name].filter(Boolean).join(' ') || 'Your account'}</strong>
            <span>{user.phone || user.email}</span>
          </div>
        </div>
        <nav className="account__nav" aria-label="Account">
          {ITEMS.map(({ to, label, icon: Icon, end }) => (
            <NavLink key={to} to={to} end={end} className="account__link">
              <Icon aria-hidden="true" />
              {label}
            </NavLink>
          ))}
          <NavLink to="/data" end className="account__link">
            <LuSmartphone aria-hidden="true" />
            Buy data
          </NavLink>
          <button type="button" className="account__link account__signout" onClick={out}>
            <LuLogOut aria-hidden="true" />
            Sign out
          </button>
        </nav>
      </aside>
      <section className="account__main">{children}</section>
    </div>
  );
}
