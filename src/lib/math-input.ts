export type MathFormat = 'number' | 'logic' | 'expression' | 'base';

export type Base = 2 | 8 | 10 | 16;

/** Normalise Arabic-Indic digits and separators to ASCII. */
export function asciiDigits(value: string): string {
  return value
    .replace(/[٠-٩]/g, (digit) => String('٠١٢٣٤٥٦٧٨٩'.indexOf(digit)))
    .replace(/[۰-۹]/g, (digit) => String('۰۱۲۳۴۵۶۷۸۹'.indexOf(digit)))
    .replace(/٫/g, '.')
    .replace(/[,٬]/g, '');
}

const COMMAND_SPACE = /\\(?:left(?![a-zA-Z])|right(?![a-zA-Z])|!|,|;|:|quad|qquad|displaystyle|textstyle)\s*/g;

function stripWrappers(source: string): string {
  let value = asciiDigits(source.trim());
  value = value
    .replace(/^\s*\$\$?/, '')
    .replace(/\$\$?\s*$/, '')
    .replace(/^\s*\\\[/, '')
    .replace(/\\\]\s*$/, '')
    .replace(/^\s*\\\(/, '')
    .replace(/\\\)\s*$/, '');
  value = value.replace(COMMAND_SPACE, '');
  value = value.replace(/\\(?:mathrm|text|operatorname|textrm)\{([^{}]*)\}/g, '$1');
  value = value.replace(/\\operatorname\*\{([^{}]*)\}/g, '$1');
  value = value.replace(/\\left\./g, '').replace(/\\right\./g, '');
  return value;
}

/** Read a balanced `{...}` group starting at `start` (which must be `{`). */
function readGroup(source: string, start: number): { body: string; end: number } | null {
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

function readArgument(source: string, start: number): { body: string; end: number } | null {
  if (source[start] === '{') return readGroup(source, start);
  if (start >= source.length) return null;
  return { body: source[start], end: start + 1 };
}

function isAtom(value: string): boolean {
  return /^[+-]?(?:\d+(?:\.\d+)?|\d*\.\d+)$/.test(value.trim());
}

function fractionToPlain(numerator: string, denominator: string): string {
  const top = numerator.trim();
  const bottom = denominator.trim();
  if (isAtom(top) && isAtom(bottom)) return `${top}/${bottom}`;
  return `(${top})/(${bottom})`;
}

function replaceFractions(source: string): string {
  let value = source;
  let guard = 0;
  while (guard < 50) {
    guard += 1;
    const match = /\\[dt]?frac\s*/.exec(value);
    if (!match) break;
    const start = match.index;
    const top = readArgument(value, start + match[0].length);
    if (!top) break;
    const bottom = readArgument(value, top.end);
    if (!bottom) break;
    value =
      value.slice(0, start) +
      fractionToPlain(replaceFractions(top.body), replaceFractions(bottom.body)) +
      value.slice(bottom.end);
  }
  return value;
}

function replacePowers(source: string): string {
  let value = source;
  let guard = 0;
  while (guard < 30) {
    guard += 1;
    const match = /\^\s*/.exec(value);
    if (!match) break;
    const start = match.index;
    const argument = readArgument(value, start + match[0].length);
    if (!argument) break;
    const body = argument.body.trim();
    value =
      value.slice(0, start) +
      `^${isAtom(body) ? body : `(${body})`}` +
      value.slice(argument.end);
  }
  return value;
}

const LOGIC_COMMANDS: [RegExp, string][] = [
  [/\\leftrightarrow(?![a-zA-Z])|\\Leftrightarrow(?![a-zA-Z])|\\iff(?![a-zA-Z])/g, '<->'],
  [/\\rightarrow(?![a-zA-Z])|\\Rightarrow(?![a-zA-Z])|\\implies(?![a-zA-Z])|\\to(?![a-zA-Z])/g, '->'],
  [/\\oplus(?![a-zA-Z])|\\veebar(?![a-zA-Z])/g, '^'],
  [/\\land(?![a-zA-Z])|\\wedge(?![a-zA-Z])/g, '&'],
  [/\\lor(?![a-zA-Z])|\\vee(?![a-zA-Z])/g, '|'],
  [/\\neg(?![a-zA-Z])|\\lnot(?![a-zA-Z])/g, '~'],
  [/\\top(?![a-zA-Z])/g, '1'],
  [/\\bot(?![a-zA-Z])/g, '0'],
  [/\\mid(?![a-zA-Z])/g, '|']
];

const LOGIC_UNICODE: [RegExp, string][] = [
  [/↔|⇔/g, '<->'],
  [/→|⇒/g, '->'],
  [/⊕/g, '^'],
  [/∧/g, '&'],
  [/∨/g, '|'],
  [/¬/g, '~'],
  [/⊤/g, '1'],
  [/⊥/g, '0'],
  [/−/g, '-']
];

function cleanLatex(source: string): string {
  return replacePowers(replaceFractions(stripWrappers(source)));
}

function cleanPunctuation(value: string): string {
  return asciiDigits(value)
    .replace(/\\[,;:!]/g, '')
    .replace(/\\ /g, '')
    .replace(/[{}]/g, '')
    .replace(/\s+/g, '')
    .replace(/−/g, '-')
    .replace(/×/g, '*')
    .replace(/÷/g, '/');
}

export function latexToNumber(latex: string): string {
  let value = cleanLatex(latex);
  value = value
    .replace(/\\sqrt\s*\{([^{}]*)\}/g, 'sqrt($1)')
    .replace(/\\sqrt\s*([0-9.]+)/g, 'sqrt($1)')
    .replace(/\\cdot|\\times|\\ast/g, '*')
    .replace(/\\div/g, '/')
    .replace(/\\pi/g, 'pi');
  value = cleanPunctuation(value);
  const evaluated = tryEvaluateExact(value);
  if (evaluated !== null) {
    value = evaluated;
  } else if (/sqrt/.test(value)) {
    const approximated = approximateRoots(value);
    const result = tryEvaluateExact(approximated);
    value = result ?? approximated;
  }
  // Remove redundant outer parentheses around a single atom.
  while (/^\([^()]+\)$/.test(value)) value = value.slice(1, -1);
  // A leading plus is noise for the solvers.
  value = value.replace(/^\+/, '');
  return value;
}

export function latexToLogic(latex: string): string {
  let value = asciiDigits(stripWrappers(latex));
  for (const [pattern, replacement] of LOGIC_COMMANDS) value = value.replace(pattern, replacement);
  for (const [pattern, replacement] of LOGIC_UNICODE) value = value.replace(pattern, replacement);
  value = value.replace(/\\mathrm\{([^{}]*)\}/g, '$1').replace(/\\text\{([^{}]*)\}/g, '$1');
  value = value.replace(/[{}]/g, '').replace(/\s+/g, '');
  value = value.replace(/−/g, '-');
  return value;
}

function replaceLatexFractions(source: string): string {
  let value = source;
  let guard = 0;
  while (guard < 50) {
    guard += 1;
    const match = /\\[dt]?frac\s*/.exec(value);
    if (!match) break;
    const start = match.index;
    const top = readArgument(value, start + match[0].length);
    if (!top) break;
    const bottom = readArgument(value, top.end);
    if (!bottom) break;
    value =
      value.slice(0, start) +
      `(${replaceLatexFractions(top.body)})/(${replaceLatexFractions(bottom.body)})` +
      value.slice(bottom.end);
  }
  return value;
}

function replaceLatexSqrts(source: string): string {
  let value = source;
  let guard = 0;
  while (guard < 50) {
    guard += 1;
    const match = /\\sqrt\s*/.exec(value);
    if (!match) break;
    const start = match.index;
    const argumentStart = start + match[0].length;
    if (value[argumentStart] === '[') {
      break;
    }
    const argument = readArgument(value, argumentStart);
    if (!argument) break;
    value =
      value.slice(0, start) +
      `sqrt(${replaceLatexSqrts(argument.body)})` +
      value.slice(argument.end);
  }
  return value;
}

function replaceLatexSuperscripts(source: string): string {
  let value = source;
  let guard = 0;
  while (guard < 50) {
    guard += 1;
    const match = /\^\s*\{/.exec(value);
    if (!match) break;
    const start = match.index;
    const brace = start + match[0].length - 1;
    const group = readGroup(value, brace);
    if (!group) break;
    value =
      value.slice(0, start) +
      `^(${replaceLatexSuperscripts(group.body)})` +
      value.slice(group.end);
  }
  return value;
}

/** Convert a LaTeX expression to the plain syntax used by the series parser. */
export function latexToExpression(latex: string): string {
  let value = stripWrappers(latex);
  value = value.replace(/\\left\s*/g, '').replace(/\\right\s*/g, '');
  value = replaceLatexFractions(value);
  value = replaceLatexSqrts(value);
  value = replaceLatexSuperscripts(value);
  value = value
    .replace(/\\(?:cdot|times|ast)\b/g, '*')
    .replace(/\\div\b/g, '/')
    .replace(/\\pi\b/g, 'pi')
    .replace(/\\(sinh|cosh|sin|cos|tan|exp|ln)\b/g, '$1')
    .replace(/[{}]/g, '');
  return value.replace(/\s+/g, ' ').trim();
}

/** Render the plain series syntax back into LaTeX for MathLive. */
export function expressionToLatex(value: string): string {
  return asciiDigits(value.trim())
    .replace(/\b(sinh|cosh|sin|cos|tan|exp|ln)\b/g, '\\$1')
    .replace(/\bpi\b/g, '\\pi')
    .replace(/sqrt\(([^()]*)\)/g, '\\sqrt{$1}')
    .replace(/\*/g, '\\cdot ');
}

export function latexToBase(latex: string, _base: Base): string {
  let value = asciiDigits(stripWrappers(latex));
  value = value.replace(/\\mathrm\{([^{}]*)\}/g, '$1').replace(/\\text\{([^{}]*)\}/g, '$1');
  value = value.replace(/[{}]/g, '').replace(/\s+/g, '').replace(/−/g, '-');
  return value;
}

export function latexToCanonical(latex: string, format: MathFormat, base: Base = 10): string {
  if (format === 'logic') return latexToLogic(latex);
  if (format === 'expression') return latexToExpression(latex);
  if (format === 'base') return latexToBase(latex, base);
  return latexToNumber(latex);
}

export function numberToLatex(value: string): string {
  const trimmed = asciiDigits(value.trim());
  if (!trimmed) return '';
  const fraction = /^([+-]?)(\d+)\/(\d+)$/.exec(trimmed);
  if (fraction) {
    const sign = fraction[1] === '-' ? '-' : '';
    return `${sign}\\frac{${fraction[2]}}{${fraction[3]}}`;
  }
  return trimmed.replace(/−/g, '-');
}

export function logicToLatex(value: string): string {
  if (!value.trim()) return '';
  return asciiDigits(value)
    .replace(/<->|<=>/g, '\\leftrightarrow ')
    .replace(/->|=>/g, '\\to ')
    .replace(/~/g, '\\neg ')
    .replace(/&/g, '\\land ')
    .replace(/\|/g, '\\lor ')
    .replace(/\^/g, '\\oplus ')
    .replace(/\s+/g, ' ')
    .trim();
}

export function canonicalToLatex(value: string, format: MathFormat, _base: Base = 10): string {
  if (format === 'logic') return logicToLatex(value);
  if (format === 'expression') return expressionToLatex(value);
  if (format === 'base') return value.trim().replace(/[A-F]/g, (letter) => `\\mathrm{${letter}}`);
  return numberToLatex(value);
}


/** Convert a label produced by the logic engine (Unicode or ASCII) to LaTeX. */
export function logicLabelToLatex(value: string): string {
  return logicToLatex(latexToLogic(value));
}

/* ------------------------------------------------------------------ */
/* Exact evaluation of simple numeric expressions typed into a field. */
/* ------------------------------------------------------------------ */

interface Exact {
  n: bigint;
  d: bigint;
}

function gcd(a: bigint, b: bigint): bigint {
  let x = a < 0n ? -a : a;
  let y = b < 0n ? -b : b;
  while (y) {
    const t = x % y;
    x = y;
    y = t;
  }
  return x || 1n;
}

function exact(n: bigint, d: bigint): Exact {
  if (d === 0n) throw new Error('division by zero');
  const sign = d < 0n ? -1n : 1n;
  const g = gcd(n, d);
  return { n: (sign * n) / g, d: (sign * d) / g };
}

function addExact(a: Exact, b: Exact): Exact {
  return exact(a.n * b.d + b.n * a.d, a.d * b.d);
}

function subExact(a: Exact, b: Exact): Exact {
  return exact(a.n * b.d - b.n * a.d, a.d * b.d);
}

function mulExact(a: Exact, b: Exact): Exact {
  return exact(a.n * b.n, a.d * b.d);
}

function divExact(a: Exact, b: Exact): Exact {
  return exact(a.n * b.d, a.d * b.n);
}

function powExact(base: Exact, exponent: bigint): Exact {
  if (exponent < 0n) return divExact(exact(1n, 1n), powExact(base, -exponent));
  let result = exact(1n, 1n);
  let factor = base;
  let power = exponent;
  while (power > 0n) {
    if (power & 1n) result = mulExact(result, factor);
    factor = mulExact(factor, factor);
    power >>= 1n;
  }
  return result;
}

function integerSqrt(value: bigint): bigint | null {
  if (value < 0n) return null;
  if (value < 2n) return value;
  let low = 1n;
  let high = value;
  while (low <= high) {
    const mid = (low + high) / 2n;
    const square = mid * mid;
    if (square === value) return mid;
    if (square < value) low = mid + 1n;
    else high = mid - 1n;
  }
  return null;
}

function sqrtExact(value: Exact): Exact | null {
  const n = integerSqrt(value.n);
  const d = integerSqrt(value.d);
  if (n === null || d === null) return null;
  return exact(n, d);
}

class NumberParser {
  private index = 0;
  constructor(private readonly source: string) {}

  parse(): Exact {
    const value = this.expression();
    this.skip();
    if (this.index < this.source.length) throw new Error('unexpected token');
    return value;
  }

  private skip(): void {
    while (this.source[this.index] === ' ') this.index += 1;
  }

  private peek(): string {
    return this.source[this.index] ?? '';
  }

  private consume(character: string): void {
    this.skip();
    if (this.source[this.index] !== character) throw new Error('expected ' + character);
    this.index += 1;
  }

  private expression(): Exact {
    let value = this.term();
    while (true) {
      this.skip();
      const character = this.peek();
      if (character === '+') {
        this.index += 1;
        value = addExact(value, this.term());
      } else if (character === '-') {
        this.index += 1;
        value = subExact(value, this.term());
      } else {
        return value;
      }
    }
  }

  private term(): Exact {
    let value = this.unary();
    while (true) {
      this.skip();
      const character = this.peek();
      if (character === '*') {
        this.index += 1;
        value = mulExact(value, this.unary());
      } else if (character === '/') {
        this.index += 1;
        value = divExact(value, this.unary());
      } else {
        return value;
      }
    }
  }

  private unary(): Exact {
    this.skip();
    if (this.peek() === '-') {
      this.index += 1;
      const value = this.unary();
      return exact(-value.n, value.d);
    }
    if (this.peek() === '+') {
      this.index += 1;
      return this.unary();
    }
    return this.power();
  }

  private power(): Exact {
    const base = this.primary();
    this.skip();
    if (this.peek() === '^') {
      this.index += 1;
      const exponent = this.unary();
      if (exponent.d !== 1n) throw new Error('fractional exponent');
      if (exponent.n > 1000n || exponent.n < -1000n) throw new Error('exponent too large');
      return powExact(base, exponent.n);
    }
    return base;
  }

  private primary(): Exact {
    this.skip();
    if (this.peek() === '(') {
      this.index += 1;
      const value = this.expression();
      this.consume(')');
      return value;
    }
    if (this.source.startsWith('sqrt', this.index)) {
      this.index += 4;
      this.consume('(');
      const value = this.expression();
      this.consume(')');
      const root = sqrtExact(value);
      if (!root) throw new Error('irrational root');
      return root;
    }
    const match = /^\d+(?:\.\d+)?/.exec(this.source.slice(this.index));
    if (!match) throw new Error('expected a number');
    this.index += match[0].length;
    if (match[0].includes('.')) {
      const [whole, fraction = ''] = match[0].split('.');
      return exact(BigInt(`${whole}${fraction}`), 10n ** BigInt(fraction.length));
    }
    return exact(BigInt(match[0]), 1n);
  }
}

function approximateRoots(value: string): string {
  return value.replace(/sqrt\(([^()]*)\)/g, (_match, inner: string) => {
    const exact = tryEvaluateExact(inner);
    let number: number;
    if (exact !== null) {
      number = Number(exact);
    } else if (inner.includes('/')) {
      const [numerator, denominator] = inner.split('/');
      number = Number(numerator) / Number(denominator);
    } else {
      number = Number(inner);
    }
    if (!Number.isFinite(number) || number < 0) return inner;
    return Number(Math.sqrt(number).toFixed(9)).toString();
  });
}

function tryEvaluateExact(value: string): string | null {
  const operatorAfterSign = /[+\-]/.test(value.slice(1));
  if (!/sqrt|\^|[()*]/.test(value) && !operatorAfterSign) return null;
  try {
    const result = new NumberParser(value).parse();
    return result.d === 1n ? result.n.toString() : `${result.n}/${result.d}`;
  } catch {
    return null;
  }
}
