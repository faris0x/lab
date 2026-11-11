import { readable } from 'svelte/store';

type Lang = 'en' | 'ar';

function initial(): Lang {
  if (typeof document === 'undefined') return 'en';
  return document.documentElement.lang === 'ar' ? 'ar' : 'en';
}

export const lang = readable<Lang>(initial(), (set) => {
  if (typeof document === 'undefined') return;
  const handler = (event: Event) => {
    const detail = (event as CustomEvent<{ lang: Lang }>).detail;
    set(detail?.lang === 'ar' ? 'ar' : 'en');
  };
  document.addEventListener('langchange', handler);
  return () => document.removeEventListener('langchange', handler);
});
