import { NavLink } from '../../router';
import { useAuth } from '../../context/Auth';
import { tabBarItems } from './nav';

/** Phone-only bottom navigation for the five most used destinations. */
export default function TabBar() {
  const { user } = useAuth();
  return (
    <nav className="tabbar" aria-label="Quick links">
      {tabBarItems(!!user).map(({ to, label, icon: Icon, end }) => (
        <NavLink key={to} to={to} end={end} className="tabbar__item">
          <Icon aria-hidden="true" />
          <span>{label}</span>
        </NavLink>
      ))}
    </nav>
  );
}
