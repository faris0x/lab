// Copyright (c) 2026 Faris Alfarhan
// SPDX-License-Identifier: GPL-3.0-only

import katex from 'katex';

/** Build-time KaTeX for static pages. Never ships KaTeX JS to the browser. */
export function tex(source: string, displayMode = false): string {
  return katex.renderToString(source, {
    displayMode,
    throwOnError: false,
    strict: 'ignore'
  });
}
