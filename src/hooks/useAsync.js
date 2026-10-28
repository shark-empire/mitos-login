import { useCallback, useEffect, useRef, useState } from 'react';

/** Runs an async loader on mount (and when `deps` change). status: loading | ready | error. */
export function useAsync(loader, deps = []) {
  const [state, setState] = useState({ data: null, status: 'loading' });
  const run = useRef(loader);
  run.current = loader;
  const [tick, setTick] = useState(0);

  useEffect(() => {
    let off = false;
    setState((s) => ({ data: s.data, status: 'loading' }));
    Promise.resolve()
      .then(() => run.current())
      .then((data) => !off && setState({ data, status: 'ready' }))
      .catch(() => !off && setState({ data: null, status: 'error' }));
    return () => {
      off = true;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [...deps, tick]);

  const reload = useCallback(() => setTick((n) => n + 1), []);
  return { ...state, reload };
}
