import { useEffect } from 'react';
import { site } from '../config/site';
import profile from '../config/profile';

export function usePageTitle(title) {
  useEffect(() => {
    // The home page keeps the exact title written into index.html, so what search engines read before and after the app starts is the same.
    document.title = title ? `${title} | ${site.name}` : profile.title;
  }, [title]);
}
