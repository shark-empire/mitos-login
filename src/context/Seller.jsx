import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react';
import { storesApi } from '../services/rest';
import { config } from '../config/env';
import { session } from '../lib/storage';
import { loginError, signupError } from '../lib/authErrors';

// Store owners sign in to the Store project, which is a different Supabase project from the customer accounts.
// Their session is kept separately so signing in as one never signs you out of the other.
const SellerContext = createContext(null);

const TOKEN_KEY = 'sf_seller_token';
const REFRESH_KEY = 'sf_seller_refresh';

const toSeller = (authUser, token) => ({
  id: authUser.id,
  email: authUser.email,
  name: authUser.user_metadata?.full_name || '',
  phone: authUser.user_metadata?.phone || '',
  token,
});

export function SellerProvider({ children }) {
  const [seller, setSeller] = useState(null);
  const [ready, setReady] = useState(!config.stores.enabled);
  const tokenRef = useRef(null);

  const store = useCallback((token, refresh) => {
    tokenRef.current = token;
    session.set(TOKEN_KEY, token);
    if (refresh) session.set(REFRESH_KEY, refresh);
  }, []);

  const clear = useCallback(() => {
    tokenRef.current = null;
    session.remove(TOKEN_KEY);
    session.remove(REFRESH_KEY);
    setSeller(null);
  }, []);

  const refresh = useCallback(async () => {
    const r = session.get(REFRESH_KEY);
    if (!r) return null;
    try {
      const d = await storesApi.auth.refresh(r);
      store(d.access_token, d.refresh_token);
      return d.access_token;
    } catch {
      return null;
    }
  }, [store]);

  useEffect(() => {
    if (!config.stores.enabled) return undefined;
    let off = false;
    (async () => {
      let token = session.get(TOKEN_KEY);
      if (!token) return;
      try {
        let u;
        try {
          u = await storesApi.auth.user(token);
        } catch (err) {
          if (err.status !== 401 && err.status !== 403) throw err;
          token = await refresh();
          if (!token) throw err;
          u = await storesApi.auth.user(token);
        }
        if (!off) {
          tokenRef.current = token;
          setSeller(toSeller(u, token));
        }
      } catch {
        if (!off) clear();
      }
    })().finally(() => !off && setReady(true));
    return () => {
      off = true;
    };
  }, [refresh, clear]);

  const signIn = useCallback(
    async (email, password) => {
      let d;
      try {
        d = await storesApi.auth.signIn(email.trim(), password);
      } catch (err) {
        throw Object.assign(new Error(loginError(err, 'email', email.trim())), { status: err.status, code: err.code });
      }
      store(d.access_token, d.refresh_token);
      const next = toSeller(d.user, d.access_token);
      setSeller(next);
      return next;
    },
    [store],
  );

  const signUp = useCallback(
    async ({ fullName, phone, email, password }) => {
      let d;
      try {
        d = await storesApi.auth.signUp(email.trim(), password, { full_name: fullName, phone: phone || '' });
      } catch (err) {
        throw Object.assign(new Error(signupError(err, 'email')), { status: err.status, code: err.code });
      }
      // With email confirmation switched on there is no session until the link is opened. (Supabase then answers
      // with the bare user, and an address that is already registered has no identities.)
      if (!d.access_token) {
        const created = d.user || d;
        if (Array.isArray(created.identities) && created.identities.length === 0) {
          throw new Error(signupError({ code: 'user_already_exists' }, 'email'));
        }
        return { needsConfirmation: true };
      }
      store(d.access_token, d.refresh_token);
      const next = toSeller(d.user || { id: '', email }, d.access_token);
      setSeller(next);
      return { seller: next };
    },
    [store],
  );

  /** Run a call with the current token; if it has expired, refresh once and retry, then sign out. */
  const authed = useCallback(
    async (fn) => {
      try {
        return await fn(tokenRef.current);
      } catch (err) {
        if (err.status === 401) {
          const fresh = await refresh();
          if (fresh) return fn(fresh);
          clear();
        }
        throw err;
      }
    },
    [refresh, clear],
  );

  const value = useMemo(() => ({ seller, ready, signIn, signUp, signOut: clear, authed }), [seller, ready, signIn, signUp, clear, authed]);
  return <SellerContext.Provider value={value}>{children}</SellerContext.Provider>;
}

export const useSeller = () => useContext(SellerContext);
