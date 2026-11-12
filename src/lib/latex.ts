const SPECIAL: Record<string, string> = {
  '\\': '\\textbackslash{}',
  '&': '\\&',
  '%': '\\%',
  '$': '\\$',
  '#': '\\#',
  '_': '\\_',
  '{': '\\{',
  '}': '\\}',
  '~': '\\textasciitilde{}',
  '^': '\\textasciicircum{}'
};

/** Escape text that goes into a LaTeX document outside a maths environment. */
export function escapeLatex(value: string): string {
  return String(value).replace(/[\\&%$#_{}~^]/g, (character) => SPECIAL[character]);
}

export interface LatexSection {
  title?: string;
  body: string;
}

export interface LatexDocumentOptions {
  title: string;
  subtitle?: string;
  sections: LatexSection[];
  preamble?: string[];
}

export function latexDocument({
  title,
  subtitle,
  sections,
  preamble = []
}: LatexDocumentOptions): string {
  const packages = [
    '\\usepackage[a4paper,margin=2.2cm]{geometry}',
    '\\usepackage{amsmath,amssymb}',
    ...preamble
  ].join('\n');
  const body = sections
    .map((section) => {
      const heading = section.title ? `\\subsection*{${escapeLatex(section.title)}}\n` : '';
      return `${heading}${section.body.trim()}`;
    })
    .join('\n\n');

  return `\\documentclass[11pt]{article}
${packages}
\\setlength{\\parindent}{0pt}
\\begin{document}
\\section*{${escapeLatex(title)}}
${subtitle ? `${escapeLatex(subtitle)}\n` : ''}
${body}
\\end{document}
`;
}

/** Render rows as a LaTeX array. Cells are already-LaTeX strings. */
export function arrayLatex(rows: string[][], columnSpec?: string): string {
  const spec = columnSpec ?? rows[0]?.map(() => 'c').join('') ?? 'c';
  return `\\begin{array}{${spec}}\n${rows
    .map((row) => row.join(' & '))
    .join(' \\\\\n')}\n\\end{array}`;
}

/** Render rows as a bracketed matrix. Cells are already-LaTeX strings. */
export function matrixLatex(rows: string[][], environment: 'bmatrix' | 'pmatrix' | 'vmatrix' = 'bmatrix'): string {
  return `\\begin{${environment}}\n${rows
    .map((row) => row.join(' & '))
    .join(' \\\\\n')}\n\\end{${environment}}`;
}

/** A matrix with a vertical separator, used for augmented systems. */
export function augmentedMatrixLatex(
  left: string[][],
  right: string[][],
  environment: 'bmatrix' | 'pmatrix' = 'bmatrix'
): string {
  const rows = left.map((row, index) => [...row, `\\mid`, ...(right[index] ?? [])]);
  return matrixLatex(rows, environment);
}

export function displayMath(source: string): string {
  return `\\[\n${source}\n\\]`;
}

export function math(source: string): string {
  return `$${source}$`;
}

/** Normalise a list of LaTeX fragments into one aligned display block. */
export function aligned(latexLines: string[]): string {
  return `\\begin{aligned}\n${latexLines.join(' \\\\\n')}\n\\end{aligned}`;
}
