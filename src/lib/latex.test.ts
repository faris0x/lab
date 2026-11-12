import { describe, expect, it } from 'vitest';
import {
  arrayLatex,
  augmentedMatrixLatex,
  escapeLatex,
  latexDocument,
  matrixLatex
} from './latex';

describe('escapeLatex', () => {
  it('escapes text specials', () => {
    expect(escapeLatex('a_b & c')).toBe('a\\_b \\& c');
    expect(escapeLatex('50%')).toBe('50\\%');
  });
});

describe('matrix helpers', () => {
  it('renders an array', () => {
    expect(arrayLatex([['1', '2']])).toBe(
      '\\begin{array}{cc}\n1 & 2\n\\end{array}'
    );
  });

  it('renders a matrix', () => {
    expect(matrixLatex([['1'], ['2']], 'pmatrix')).toBe(
      '\\begin{pmatrix}\n1 \\\\\n2\n\\end{pmatrix}'
    );
  });

  it('renders an augmented matrix', () => {
    expect(augmentedMatrixLatex([['1']], [['2']])).toBe(
      '\\begin{bmatrix}\n1 & \\mid & 2\n\\end{bmatrix}'
    );
  });
});

describe('latexDocument', () => {
  it('builds a complete document', () => {
    const document = latexDocument({
      title: 'Test & check',
      sections: [{ title: 'Working', body: 'x = 1' }]
    });
    expect(document).toContain('\\documentclass[11pt]{article}');
    expect(document).toContain('\\section*{Test \\& check}');
    expect(document).toContain('\\subsection*{Working}');
    expect(document.trimEnd().endsWith('\\end{document}')).toBe(true);
  });
});
