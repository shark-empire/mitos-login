// The stores-only site (VITE_SITE_MODE=stores). It has no accounts, no marketplace and none of the main site's pages:
// a customer can open a store link, buy, and track the order. Everything else shows a blank notice.
import { useEffect } from 'react';
import { Router, useLocation } from './router';
import { ThemeProvider } from './context/Theme';
import { ToastProvider } from './context/Toast';
import LiveFX from './components/LiveFX';
import Shop from './pages/stores/Shop';
import { matchShop } from './pages/stores/shopRoutes';

function Nothing() {
  useEffect(() => {
    document.title = 'Shop';
  }, []);
  return (
    <main id="main" tabIndex={-1} className="shopnotice">
      <p>Open the link your store gave you to shop.</p>
    </main>
  );
}

function Routes() {
  const { path } = useLocation();
  const shop = matchShop(path, { legacy: true });
  return shop ? <Shop key={shop.slug} {...shop} /> : <Nothing />;
}

export default function StoresApp() {
  return (
    <ThemeProvider>
      <LiveFX />
      <ToastProvider>
        <Router>
          <Routes />
        </Router>
      </ToastProvider>
    </ThemeProvider>
  );
}
