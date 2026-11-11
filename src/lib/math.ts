import { arNumeric, localizeLatex } from './localize';

type KatexLike = {
  renderToString(
    source: string,
    options?: {
      displayMode?: boolean;
      throwOnError?: boolean;
      strict?: string;
    }
  ): string;
};

let katexPromise: Promise<KatexLike> | null = null;
const cache = new Map<string, string>();

function loadKatex(): Promise<KatexLike> {
  if (!katexPromise) {
    katexPromise = (async () => {
      const mod = await import('katex');
      return ((mod as unknown as { default?: KatexLike }).default ??
        (mod as unknown as KatexLike)) as KatexLike;
    })().catch((error) => {
      katexPromise = null;
      throw error;
    });
  }
  return katexPromise;
}

/** Start fetching KaTeX without blocking the current interaction. */
export function preloadMath(): void {
  void loadKatex().catch(() => {});
}

function arabic(): boolean {
  return typeof document !== 'undefined' && document.documentElement.lang === 'ar';
}

/** Convert digits in KaTeX text nodes without touching HTML entities or attributes. */
function arabicHtml(html: string): string {
  return html.replace(/>([^<>]*)</g, (_match, text: string) => {
    const converted = text
      .split(/(&#?[a-zA-Z0-9]+;)/g)
      .map((part, index) => (index % 2 === 1 ? part : arNumeric(part)))
      .join('');
    return `>${converted}<`;
  });
}

export async function renderMath(
  source: string,
  displayMode = false
): Promise<string> {
  const isArabic = arabic();
  const key = `${isArabic ? 'ar' : 'en'}:${displayMode ? 'd' : 'i'}:${source}`;
  const hit = cache.get(key);
  if (hit) return hit;
  const katex = await loadKatex();
  const html = katex.renderToString(
    isArabic ? localizeLatex(source) : source,
    {
      displayMode,
      throwOnError: false,
      strict: 'ignore'
    }
  );
  const output = isArabic ? arabicHtml(html) : html;
  cache.set(key, output);
  return output;
}

function escapeHtml(value: string): string {
  return value.replace(/[&<>"']/g, (character) => {
    switch (character) {
      case '&':
        return '&amp;';
      case '<':
        return '&lt;';
      case '>':
        return '&gt;';
      case '"':
        return '&quot;';
      default:
        return '&#39;';
    }
  });
}

/** Render prose containing inline `$...$` fragments. */
export async function renderMathText(text: string): Promise<string> {
  const parts = text.split(/(\$[^$]+\$)/g);
  const rendered = await Promise.all(
    parts.map(async (part) => {
      if (part.length > 2 && part.startsWith('$') && part.endsWith('$')) {
        try {
          return await renderMath(part.slice(1, -1), false);
        } catch {
          return escapeHtml(part);
        }
      }
      return escapeHtml(part);
    })
  );
  return rendered.join('');
}
