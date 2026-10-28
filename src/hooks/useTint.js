import { useEffect } from 'react';

/**
 * Sets the colour of the glass for as long as the calling page is on screen.
 * Pass 'mtn' | 'telecel' | 'airteltigo' | 'results', or nothing for the house colour.
 */
export function useTint(name) {
  useEffect(() => {
    const root = document.documentElement;
    if (name) root.dataset.tint = name;
    else delete root.dataset.tint;
    return () => {
      delete root.dataset.tint;
    };
  }, [name]);
}
