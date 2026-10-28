import { LuGraduationCap, LuHouse, LuPackage, LuSmartphone, LuUser } from 'react-icons/lu';
import { config } from '../../config/env';

export const MAIN_NAV = [
  { to: '/data', label: 'Data bundles' },
  { to: '/results', label: 'Results checkers' },
  // The marketplace only appears once the Store backend is connected.
  ...(config.stores.enabled ? [{ to: '/marketplace', label: 'Marketplace' }] : []),
  { to: '/track', label: 'Track order' },
  { to: '/how-it-works', label: 'How it works' },
  { to: '/developers', label: 'Developers' },
];

export const tabBarItems = (signedIn) => [
  { to: '/', label: 'Home', icon: LuHouse, end: true },
  { to: '/data', label: 'Data', icon: LuSmartphone },
  { to: '/results', label: 'Results', icon: LuGraduationCap },
  { to: '/track', label: 'Track', icon: LuPackage },
  signedIn ? { to: '/account', label: 'Account', icon: LuUser } : { to: '/login', label: 'Log in', icon: LuUser },
];
