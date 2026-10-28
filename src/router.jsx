// A small hash router. Hash URLs (#/data) work on any static host with zero server config,
// including GitHub Pages project sites, so the build is deployable anywhere as-is.
import { createContext, useCallback, useContext, useEffect, useMemo, useState } from 'react';

const RouterContext = createContext(null);

function parseHash() {
  const raw = window.location.hash.replace(/^#/, '') || '/';
  const [pathPart, query = ''] = raw.split('?');
  let path = pathPart.startsWith('/') ? pathPart : `/${pathPart}`;
  if (path.length > 1) path = path.replace(/\/+$/, '');
  return { path, search: new URLSearchParams(query) };
}

export function Router({ children }) {
  const [location, setLocation] = useState(parseHash);

  // Keep the same object when nothing changed, so a hashchange that follows navigate() is a no-op.
  const sync = useCallback(() => {
    setLocation((prev) => {
      const next = parseHash();
      return prev.path === next.path && prev.search.toString() === next.search.toString() ? prev : next;
    });
  }, []);

  useEffect(() => {
    const onChange = () => sync();
    window.addEventListener('hashchange', onChange);
    return () => window.removeEventListener('hashchange', onChange);
  }, [sync]);

  // New page: scroll to top and move focus to the main region for keyboard / screen reader users.
  useEffect(() => {
    window.scrollTo(0, 0);
    const main = document.getElementById('main');
    if (main) main.focus({ preventScroll: true });
  }, [location.path]);

  // Updates router state immediately (not only on the async hashchange event), so code that
  // navigates and then changes other state in the same handler sees one consistent location.
  const navigate = useCallback(
    (to, { replace = false } = {}) => {
      if (replace) {
        const { pathname, search } = window.location;
        window.location.replace(`${pathname}${search}#${to}`);
      } else {
        window.location.hash = to;
      }
      sync();
    },
    [sync],
  );

  const value = useMemo(() => ({ location, navigate }), [location, navigate]);
  return <RouterContext.Provider value={value}>{children}</RouterContext.Provider>;
}

export const useLocation = () => useContext(RouterContext).location;
export const useNavigate = () => useContext(RouterContext).navigate;

export function Link({ to, children, ...rest }) {
  return (
    <a href={`#${to}`} {...rest}>
      {children}
    </a>
  );
}

/** A link that knows when it points at the current page. */
export function NavLink({ to, end = false, className = '', children, ...rest }) {
  const { path } = useLocation();
  const active = end ? path === to : path === to || path.startsWith(`${to}/`);
  return (
    <a
      href={`#${to}`}
      className={`${className}${active ? ' is-active' : ''}`.trim()}
      aria-current={active ? 'page' : undefined}
      {...rest}
    >
      {children}
    </a>
  );
}
