import { describe, expect, it } from 'vitest';
import {
  asciiDigits,
  canonicalToLatex,
  expressionToLatex,
  latexToCanonical,
  latexToExpression,
  latexToLogic,
  latexToNumber,
  logicToLatex,
  numberToLatex
} from './math-input';

describe('asciiDigits', () => {
  it('normalises Arabic-Indic and Persian digits', () => {
    expect(asciiDigits('١٢٣')).toBe('123');
    expect(asciiDigits('۴۵۶')).toBe('456');
    expect(asciiDigits('١٫٥')).toBe('1.5');
    expect(asciiDigits('1٬000')).toBe('1000');
  });
});

describe('latexToNumber', () => {
  it('unwraps maths delimiters', () => {
    expect(latexToNumber('$\\frac{1}{2}$')).toBe('1/2');
  });

  it('converts fractions and powers', () => {
    expect(latexToNumber('\\frac{3}{4}')).toBe('3/4');
    expect(latexToNumber('1^5')).toBe('1');
    expect(latexToNumber('2^3')).toBe('8');
  });

  it('evaluates simple exact expressions', () => {
    expect(latexToNumber('2+3')).toBe('5');
    expect(latexToNumber('\\sqrt{4}')).toBe('2');
  });

  it('approximates irrational roots', () => {
    expect(latexToNumber('\\sqrt{2}')).toBe('1.414213562');
  });
});

describe('logic conversion', () => {
  it('maps LaTeX operators to the canonical form', () => {
    expect(latexToLogic('p \\land q')).toBe('p&q');
    expect(latexToLogic('\\neg p \\lor q')).toBe('~p|q');
    expect(latexToLogic('p \\leftrightarrow q')).toBe('p<->q');
    expect(latexToLogic('p \\to q')).toBe('p->q');
    expect(latexToLogic('\\top')).toBe('1');
    expect(latexToLogic('\\bot')).toBe('0');
  });

  it('round-trips canonical logic to LaTeX', () => {
    expect(logicToLatex('p&q')).toBe('p\\land q');
    expect(canonicalToLatex('p<->q', 'logic')).toBe('p\\leftrightarrow q');
    expect(latexToCanonical('p \\land q', 'logic')).toBe('p&q');
  });
});

describe('expression conversion', () => {
  it('converts LaTeX functions and fractions to the series syntax', () => {
    expect(latexToExpression('\\sin\\left(x\\right)')).toBe('sin(x)');
    expect(latexToExpression('\\frac{1}{1-x}')).toBe('(1)/(1-x)');
    expect(latexToExpression('\\sqrt{x+1}')).toBe('sqrt(x+1)');
    expect(latexToExpression('e^{x+1}')).toBe('e^(x+1)');
    expect(latexToExpression('\\pi x')).toBe('pi x');
    expect(latexToExpression('2 \\cdot x')).toBe('2 * x');
  });

  it('renders the series syntax back into LaTeX', () => {
    expect(expressionToLatex('sin(x)')).toBe('\\sin(x)');
    expect(expressionToLatex('pi*x')).toBe('\\pi\\cdot x');
    expect(expressionToLatex('sqrt(x)')).toBe('\\sqrt{x}');
  });

  it('uses the expression format through the canonical helpers', () => {
    expect(latexToCanonical('\\cos\\left(x\\right)', 'expression')).toBe('cos(x)');
    expect(canonicalToLatex('cos(x)', 'expression')).toBe('\\cos(x)');
  });
});

describe('numberToLatex', () => {
  it('renders fractions and integers', () => {
    expect(numberToLatex('3/4')).toBe('\\frac{3}{4}');
    expect(numberToLatex('-3/4')).toBe('-\\frac{3}{4}');
    expect(numberToLatex('12')).toBe('12');
  });
});
