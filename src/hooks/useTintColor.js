import { useEffect } from 'react';
import { safeHex } from '../lib/color';

/** Colours the glass with an arbitrary hex (a store's accent) while the calling page is on screen. */
export function useTintColor(hex) {
  useEffect(() => {
    if (!hex) return undefined;
    const root = document.documentElement;
    root.style.setProperty('--tint', safeHex(hex));
    return () => root.style.removeProperty('--tint');
  }, [hex]);
}
