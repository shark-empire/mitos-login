import { useEffect, useState } from 'react';
import { lookupOwner } from '../services/webhooks';
import { NETWORKS } from '../config/site';

/**
 * Asks who is registered on a complete phone number, once the visitor has stopped typing.
 * state: idle | loading | ok (name) | unresolved | error. A failed lookup never blocks a purchase.
 */
export function useOwnerLookup(phone, network, complete) {
  const [owner, setOwner] = useState({ state: 'idle' });
  const known = NETWORKS.some((n) => n.id === network);

  useEffect(() => {
    if (!complete || !known) {
      setOwner({ state: 'idle' });
      return undefined;
    }
    const ctrl = new AbortController();
    setOwner({ state: 'loading' });
    const t = setTimeout(() => {
      lookupOwner({ phone, network, signal: ctrl.signal })
        .then((r) => setOwner(r.status === 'success' && r.name ? { state: 'ok', name: r.name } : { state: 'unresolved' }))
        .catch((e) => {
          if (e.name !== 'AbortError') setOwner({ state: 'error' });
        });
    }, 350);
    return () => {
      clearTimeout(t);
      ctrl.abort();
    };
  }, [phone, network, complete, known]);

  return owner;
}
