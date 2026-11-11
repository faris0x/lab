import katex from 'katex';

/** Build-time KaTeX for static pages. Never ships KaTeX JS to the browser. */
export function tex(source: string, displayMode = false): string {
  return katex.renderToString(source, {
    displayMode,
    throwOnError: false,
    strict: 'ignore'
  });
}
