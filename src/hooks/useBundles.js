import { useCallback, useEffect, useState } from 'react';
import { dataApi } from '../services/rest';
import { NETWORKS, normalizeNetwork } from '../config/site';
import { sizeInMb } from '../lib/format';

// Shared across pages so the home page rate card and the buy page use one request.
let cache = null;
let inflight = null;

function normalize(row) {
  return {
    id: row.id ?? row.bundle_id,
    size: String(row.size ?? '').trim(),
    price: Number(row.price),
    network: normalizeNetwork(row.network),
    available: row.available !== false,
  };
}

export function loadBundles({ force = false } = {}) {
  if (cache && !force) return Promise.resolve(cache);
  if (inflight) return inflight;
  inflight = dataApi
    .select('bundles', '?select=*')
    .then((rows) => {
      const order = (n) => {
        const i = NETWORKS.findIndex((x) => x.id === n);
        return i === -1 ? 99 : i;
      };
      cache = (rows || [])
        .map(normalize)
        .filter((b) => b.id != null && b.size && Number.isFinite(b.price))
        .sort((a, b) => order(a.network) - order(b.network) || sizeInMb(a.size) - sizeInMb(b.size) || a.price - b.price);
      return cache;
    })
    .finally(() => {
      inflight = null;
    });
  return inflight;
}

/** { bundles, status: 'loading' | 'ready' | 'error', reload } */
export function useBundles() {
  const [state, setState] = useState(() =>
    cache ? { bundles: cache, status: 'ready' } : { bundles: [], status: 'loading' },
  );

  const load = useCallback((force = false) => {
    setState((s) => (s.status === 'ready' && !force ? s : { ...s, status: 'loading' }));
    loadBundles({ force })
      .then((bundles) => setState({ bundles, status: 'ready' }))
      .catch(() => setState({ bundles: [], status: 'error' }));
  }, []);

  useEffect(() => {
    if (!cache) load();
  }, [load]);

  return { ...state, reload: () => load(true) };
}
