import { useEffect } from 'react';

/** Sets the browser tab title exactly as given (no site name added). */
export function useDocTitle(title) {
  useEffect(() => {
    if (title) document.title = title;
  }, [title]);
}
