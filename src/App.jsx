import { Suspense, lazy, useEffect } from 'react';
import { Router, useLocation, useNavigate } from './router';
import { ThemeProvider } from './context/Theme';
import { ToastProvider } from './context/Toast';
import { AuthProvider, useAuth } from './context/Auth';
import { SellerProvider, useSeller } from './context/Seller';
import SiteLayout from './components/layout/SiteLayout';
import AccountLayout from './components/layout/AccountLayout';
import SellerLayout from './components/layout/SellerLayout';
import { Spinner } from './components/ui';
import LiveFX from './components/LiveFX';
import Home from './pages/Home';
import BuyData from './pages/BuyData';
import NotFound from './pages/NotFound';
import { matchShop } from './pages/stores/shopRoutes';

// Everything except the landing and purchase pages loads on demand.
const DataSuccess = lazy(() => import('./pages/DataSuccess'));
const Results = lazy(() => import('./pages/Results'));
const Track = lazy(() => import('./pages/Track'));
const Guide = lazy(() => import('./pages/Guide'));
const Developers = lazy(() => import('./pages/Developers'));
const Support = lazy(() => import('./pages/Support'));
const About = lazy(() => import('./pages/About'));
const Terms = lazy(() => import('./pages/Legal').then((m) => ({ default: m.Terms })));
const Privacy = lazy(() => import('./pages/Legal').then((m) => ({ default: m.Privacy })));
const Login = lazy(() => import('./pages/Auth').then((m) => ({ default: m.Login })));
const Signup = lazy(() => import('./pages/Auth').then((m) => ({ default: m.Signup })));
const Dashboard = lazy(() => import('./pages/account/Dashboard'));
const Orders = lazy(() => import('./pages/account/Orders'));
const Wallet = lazy(() => import('./pages/account/Wallet'));
const Fund = lazy(() => import('./pages/account/Fund'));
const Settings = lazy(() => import('./pages/account/Settings'));
const Transfer = lazy(() => import('./pages/admin/Transfer'));
// Reseller stores (the Store project).
const Marketplace = lazy(() => import('./pages/stores/Marketplace'));
const Stores = lazy(() => import('./pages/stores/Stores'));
const Storefront = lazy(() => import('./pages/stores/Storefront'));
const BundleDetail = lazy(() => import('./pages/stores/BundleDetail'));
const StoreBuilder = lazy(() => import('./pages/stores/StoreBuilder'));
const SellerLogin = lazy(() => import('./pages/stores/SellerLogin'));
const SellerOverview = lazy(() => import('./pages/seller/Overview'));
const SellerOrders = lazy(() => import('./pages/seller/Orders'));
// What a store's customers see (the shop and order tracking), in a frame with no site menu.
const Shop = lazy(() => import('./pages/stores/Shop'));
const SellerBundles = lazy(() => import('./pages/seller/Bundles'));

const PUBLIC_ROUTES = {
  '/': Home,
  '/data': BuyData,
  '/data/success': DataSuccess,
  '/results': Results,
  '/track': Track,
  '/how-it-works': Guide,
  '/developers': Developers,
  '/support': Support,
  '/about': About,
  '/terms': Terms,
  '/privacy': Privacy,
  '/login': Login,
  '/signup': Signup,
  '/admin/transfer': Transfer,
  '/marketplace': Marketplace,
  '/stores': Stores,
  '/store-builder': StoreBuilder,
  '/seller/login': SellerLogin,
};

// Addresses with one variable part: /store/kwame-data-hub, /bundle/12. The page receives it as `param`.
const PARAM_ROUTES = [
  { prefix: '/store/', Page: Storefront },
  { prefix: '/bundle/', Page: BundleDetail },
];

const SELLER_ROUTES = {
  '/seller': SellerOverview,
  '/seller/orders': SellerOrders,
  '/seller/bundles': SellerBundles,
};

const ACCOUNT_ROUTES = {
  '/account': Dashboard,
  '/account/orders': Orders,
  '/account/wallet': Wallet,
  '/account/fund': Fund,
  '/account/settings': Settings,
};

const Loading = () => (
  <div className="pageloading" role="status" aria-label="Loading">
    <Spinner size={28} />
  </div>
);

/** Sends signed-out visitors to the login page and brings them back afterwards. */
function RequireAuth({ children }) {
  const { user, ready } = useAuth();
  const { path } = useLocation();
  const navigate = useNavigate();

  useEffect(() => {
    if (ready && !user) navigate(`/login?next=${encodeURIComponent(path)}`, { replace: true });
  }, [ready, user, path, navigate]);

  if (!ready || !user) return <Loading />;
  return children;
}

/** Same idea for store owners, who sign in to a separate backend. */
function RequireSeller({ children }) {
  const { seller, ready } = useSeller();
  const { path } = useLocation();
  const navigate = useNavigate();

  useEffect(() => {
    if (ready && !seller) navigate(`/seller/login?next=${encodeURIComponent(path)}`, { replace: true });
  }, [ready, seller, path, navigate]);

  if (!ready || !seller) return <Loading />;
  return children;
}

function matchParam(path) {
  for (const { prefix, Page } of PARAM_ROUTES) {
    if (path.startsWith(prefix)) {
      const rest = path.slice(prefix.length);
      if (rest && !rest.includes('/')) {
        try {
          return { Page, param: decodeURIComponent(rest) };
        } catch {
          return null;
        }
      }
    }
  }
  return null;
}

function Routes() {
  const { path } = useLocation();

  // A store's customers get the store alone: no site header, footer, menu or links.
  const shop = matchShop(path);
  if (shop) {
    return (
      <Suspense fallback={<Loading />}>
        <Shop key={shop.slug} {...shop} />
      </Suspense>
    );
  }

  const Account = ACCOUNT_ROUTES[path];
  const Seller = SELLER_ROUTES[path];
  const Page = PUBLIC_ROUTES[path];
  const withParam = !Account && !Seller && !Page ? matchParam(path) : null;

  let content;
  if (Seller) {
    content = (
      <RequireSeller>
        <SellerLayout>
          <Seller />
        </SellerLayout>
      </RequireSeller>
    );
  } else if (withParam) {
    content = <withParam.Page param={withParam.param} />;
  } else if (Account) {
    content = (
      <RequireAuth>
        <AccountLayout>
          <Account />
        </AccountLayout>
      </RequireAuth>
    );
  } else if (Page) {
    content = <Page />;
  } else {
    content = <NotFound />;
  }

  return (
    <SiteLayout>
      <Suspense fallback={<Loading />}>{content}</Suspense>
    </SiteLayout>
  );
}

export default function App() {
  return (
    <ThemeProvider>
      <LiveFX />
      <ToastProvider>
        <AuthProvider>
          <SellerProvider>
            <Router>
              <Routes />
            </Router>
          </SellerProvider>
        </AuthProvider>
      </ToastProvider>
    </ThemeProvider>
  );
}
