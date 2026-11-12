import { asciiDigits } from './math-input';

export type PreviewBlock =
  | { kind: 'heading'; level: 1 | 2 | 3; text: string }
  | { kind: 'math'; latex: string }
  | { kind: 'text'; text: string }
  | { kind: 'list'; ordered: boolean; items: PreviewBlock[][] };

function readBraced(source: string, start: number): { body: string; end: number } | null {
  if (source[start] !== '{') return null;
  let depth = 0;
  for (let index = start; index < source.length; index += 1) {
    const character = source[index];
    if (character === '\\') {
      index += 1;
      continue;
    }
    if (character === '{') depth += 1;
    else if (character === '}') {
      depth -= 1;
      if (depth === 0) return { body: source.slice(start + 1, index), end: index + 1 };
    }
  }
  return null;
}

function cleanText(value: string): string {
  return asciiDigits(value)
    .replace(/\\(?:left(?![a-zA-Z])|right(?![a-zA-Z])|!|,|;|:|quad|qquad|hfill|noindent|medskip|bigskip|smallskip)\s*/g, ' ')
    .replace(/\\\\\s*\[[^\]]*\]/g, '\n')
    .replace(/\\\\/g, '\n')
    .replace(/\\par\b/g, '\n\n')
    .replace(/\\&/g, '&')
    .replace(/\\%/g, '%')
    .replace(/\\_/g, '_')
    .replace(/\\#/g, '#')
    .replace(/\\textbackslash\{\}/g, '\\')
    .replace(/\\textasciitilde\{\}/g, '~')
    .replace(/\\textasciicircum\{\}/g, '^')
    .replace(/~/g, ' ')
    .replace(/[ \t]+/g, ' ')
    .replace(/\n{3,}/g, '\n\n')
    .trim();
}

/** Split text into paragraphs, keeping inline `$...$` maths intact. */
function textBlocks(value: string): PreviewBlock[] {
  return value
    .split(/\n\s*\n/)
    .map(cleanText)
    .filter(Boolean)
    .map((text) => ({ kind: 'text' as const, text }));
}

/** Split an item body into text and display-maths blocks. */
function inlineBlocks(value: string): PreviewBlock[] {
  const blocks: PreviewBlock[] = [];
  let buffer = '';
  let index = 0;
  const flush = () => {
    const cleaned = cleanText(buffer);
    if (cleaned) blocks.push({ kind: 'text', text: cleaned });
    buffer = '';
  };
  while (index < value.length) {
    if (value.startsWith('\\\\', index)) {
      buffer += '\\\\';
      index += 2;
      continue;
    }
    if (value.startsWith('\\[', index)) {
      const end = value.indexOf('\\]', index + 2);
      if (end === -1) {
        buffer += value.slice(index);
        break;
      }
      flush();
      blocks.push({ kind: 'math', latex: value.slice(index + 2, end).trim() });
      index = end + 2;
      continue;
    }
    buffer += value[index];
    index += 1;
  }
  flush();
  return blocks;
}

function parseList(source: string, ordered: boolean): { block: PreviewBlock; end: number } | null {
  const open = ordered ? '\\begin{enumerate}' : '\\begin{itemize}';
  const close = ordered ? '\\end{enumerate}' : '\\end{itemize}';
  if (!source.startsWith(open, 0)) return null;
  const closeIndex = source.indexOf(close);
  const content = source.slice(open.length, closeIndex === -1 ? source.length : closeIndex);
  const items = content
    .split(/\\item\b/)
    .slice(1)
    .map((item) => inlineBlocks(item.trim()))
    .filter((parts) => parts.length > 0);
  return {
    block: { kind: 'list', ordered, items },
    end: closeIndex === -1 ? source.length : closeIndex + close.length
  };
}

/** Parse the body of a generated LaTeX document into renderable blocks. */
export function parseLatexDocument(document: string): PreviewBlock[] {
  const begin = document.indexOf('\\begin{document}');
  const end = document.lastIndexOf('\\end{document}');
  const body =
    begin === -1
      ? document
      : document.slice(begin + '\\begin{document}'.length, end === -1 ? document.length : end);

  const blocks: PreviewBlock[] = [];
  let buffer = '';
  let index = 0;

  const flush = () => {
    textBlocks(buffer).forEach((block) => blocks.push(block));
    buffer = '';
  };

  while (index < body.length) {
    const rest = body.slice(index);

    // A LaTeX line break with optional spacing (`\\[4pt]`) must not be
    // mistaken for the start of display maths (`\[`).
    if (rest.startsWith('\\\\')) {
      buffer += '\\\\';
      index += 2;
      continue;
    }

    if (rest.startsWith('\\begin{itemize}') || rest.startsWith('\\begin{enumerate}')) {
      flush();
      const ordered = rest.startsWith('\\begin{enumerate}');
      const parsed = parseList(rest, ordered);
      if (!parsed) break;
      blocks.push(parsed.block);
      index += parsed.end;
      continue;
    }

    if (rest.startsWith('\\[')) {
      const close = body.indexOf('\\]', index + 2);
      if (close === -1) {
        buffer += body.slice(index);
        break;
      }
      flush();
      blocks.push({ kind: 'math', latex: body.slice(index + 2, close).trim() });
      index = close + 2;
      continue;
    }

    const headingMatch = /^\\(section|subsection|subsubsection)\*\{/.exec(rest);
    if (headingMatch) {
      flush();
      const braceStart = index + headingMatch[0].length - 1;
      const group = readBraced(body, braceStart);
      if (!group) {
        buffer += body.slice(index);
        break;
      }
      const level = headingMatch[1] === 'section' ? 1 : headingMatch[1] === 'subsection' ? 2 : 3;
      blocks.push({ kind: 'heading', level, text: cleanText(group.body) });
      index = group.end;
      continue;
    }

    buffer += body[index];
    index += 1;
  }

  flush();
  return blocks;
}
