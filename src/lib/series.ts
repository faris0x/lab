import { latexDocument } from './latex';

export type FunctionName =
  | 'sin'
  | 'cos'
  | 'tan'
  | 'exp'
  | 'ln'
  | 'sqrt'
  | 'sinh'
  | 'cosh';

export interface Rational {
  n: bigint;
  d: bigint;
}

export type Expr =
  | { kind: 'num'; value: Rational }
  | { kind: 'var' }
  | { kind: 'const'; name: 'pi' | 'e' }
  | { kind: 'add'; left: Expr; right: Expr }
  | { kind: 'sub'; left: Expr; right: Expr }
  | { kind: 'mul'; left: Expr; right: Expr }
  | { kind: 'div'; left: Expr; right: Expr }
  | { kind: 'neg'; child: Expr }
  | { kind: 'pow'; base: Expr; exponent: Rational }
  | { kind: 'fn'; name: FunctionName; arg: Expr };

export interface SeriesStep {
  k: number;
  derivativeLatex: string;
  valueLatex: string;
  coefficientLatex: string;
  coefficientPlain: string;
  termLatex: string;
}

export interface SeriesResult {
  source: string;
  functionLatex: string;
  center: string;
  centerLatex: string;
  order: number;
  kind: 'maclaurin' | 'taylor';
  seriesLatex: string;
  steps: SeriesStep[];
  summary: string[];
  summaryLatex: string[];
  latex: string;
}

export const MAX_ORDER = 12;

const ZERO: Expr = { kind: 'num', value: { n: 0n, d: 1n } };
const ONE: Expr = { kind: 'num', value: { n: 1n, d: 1n } };
const TWO: Expr = { kind: 'num', value: { n: 2n, d: 1n } };
const RAT_HALF: Rational = { n: 1n, d: 2n };
const RAT_ONE: Rational = { n: 1n, d: 1n };

function absBig(value: bigint): bigint {
  return value < 0n ? -value : value;
}

function gcdBig(a: bigint, b: bigint): bigint {
  let x = absBig(a);
  let y = absBig(b);
  while (y) {
    const t = x % y;
    x = y;
    y = t;
  }
  return x || 1n;
}

function rat(n: bigint, d = 1n): Rational {
  if (d === 0n) throw new Error('Division by zero.');
  if (d < 0n) {
    n = -n;
    d = -d;
  }
  const g = gcdBig(n, d);
  return { n: n / g, d: d / g };
}

function rAdd(a: Rational, b: Rational): Rational {
  return rat(a.n * b.d + b.n * a.d, a.d * b.d);
}

function rSub(a: Rational, b: Rational): Rational {
  return rat(a.n * b.d - b.n * a.d, a.d * b.d);
}

function rMul(a: Rational, b: Rational): Rational {
  return rat(a.n * b.n, a.d * b.d);
}

function rDiv(a: Rational, b: Rational): Rational {
  if (b.n === 0n) throw new Error('Division by zero.');
  return rat(a.n * b.d, a.d * b.n);
}

function rNeg(a: Rational): Rational {
  return { n: -a.n, d: a.d };
}

function rPow(base: Rational, exponent: bigint): Rational {
  if (base.n === 0n && exponent < 0n) {
    throw new Error('Division by zero.');
  }
  let power = exponent < 0n ? -exponent : exponent;
  let factor = exponent < 0n ? rat(base.d, base.n) : base;
  let result = rat(1n);
  while (power > 0n) {
    if (power & 1n) result = rMul(result, factor);
    factor = rMul(factor, factor);
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

function exactSqrt(value: Rational): Rational | null {
  if (value.n < 0n) return null;
  const n = integerSqrt(value.n);
  const d = integerSqrt(value.d);
  if (n === null || d === null) return null;
  return rat(n, d);
}

function isZero(value: Expr): boolean {
  return value.kind === 'num' && value.value.n === 0n;
}

function isOne(value: Expr): boolean {
  return value.kind === 'num' && value.value.n === value.value.d;
}

function isNeg(value: Expr): boolean {
  switch (value.kind) {
    case 'num':
      return value.value.n < 0n;
    case 'neg':
      return true;
    case 'mul':
      return isNeg(value.left) !== isNeg(value.right);
    case 'div':
      return isNeg(value.left) !== isNeg(value.right);
    default:
      return false;
  }
}

function key(value: Expr): string {
  switch (value.kind) {
    case 'num':
      return `n:${value.value.n}/${value.value.d}`;
    case 'var':
      return 'x';
    case 'const':
      return `c:${value.name}`;
    case 'add':
      return `a(${key(value.left)},${key(value.right)})`;
    case 'sub':
      return `s(${key(value.left)},${key(value.right)})`;
    case 'mul':
      return `m(${key(value.left)},${key(value.right)})`;
    case 'div':
      return `d(${key(value.left)},${key(value.right)})`;
    case 'neg':
      return `g(${key(value.child)})`;
    case 'pow':
      return `p(${key(value.base)},${value.exponent.n}/${value.exponent.d})`;
    case 'fn':
      return `f:${value.name}(${key(value.arg)})`;
  }
}

function same(a: Expr, b: Expr): boolean {
  return key(a) === key(b);
}

function numValue(n: bigint, d = 1n): Expr {
  return { kind: 'num', value: rat(n, d) };
}

function add(left: Expr, right: Expr): Expr {
  if (left.kind === 'num' && right.kind === 'num') {
    return { kind: 'num', value: rAdd(left.value, right.value) };
  }
  if (isZero(left)) return right;
  if (isZero(right)) return left;
  if (same(left, right)) return mul(TWO, left);
  return { kind: 'add', left, right };
}

function sub(left: Expr, right: Expr): Expr {
  if (left.kind === 'num' && right.kind === 'num') {
    return { kind: 'num', value: rSub(left.value, right.value) };
  }
  if (isZero(right)) return left;
  if (isZero(left)) return neg(right);
  if (same(left, right)) return ZERO;
  return { kind: 'sub', left, right };
}

function neg(child: Expr): Expr {
  if (child.kind === 'num') {
    return { kind: 'num', value: rNeg(child.value) };
  }
  if (child.kind === 'neg') return child.child;
  if (child.kind === 'mul' && child.left.kind === 'num') {
    return mul({ kind: 'num', value: rNeg(child.left.value) }, child.right);
  }
  return { kind: 'neg', child };
}

function mul(left: Expr, right: Expr): Expr {
  if (isZero(left) || isZero(right)) return ZERO;
  if (isOne(left)) return right;
  if (isOne(right)) return left;
  if (left.kind === 'num' && right.kind === 'num') {
    return { kind: 'num', value: rMul(left.value, right.value) };
  }
  if (right.kind === 'num' && left.kind !== 'num') return mul(right, left);
  if (left.kind === 'neg') return neg(mul(left.child, right));
  if (right.kind === 'neg') return neg(mul(left, right.child));
  if (left.kind === 'num' && right.kind === 'div') {
    return div(mul(left, right.left), right.right);
  }
  if (left.kind === 'num' && right.kind === 'mul' && right.left.kind === 'num') {
    return mul(mul(left, right.left), right.right);
  }
  if (left.kind === 'pow' && right.kind === 'pow' && same(left.base, right.base)) {
    return pow(left.base, rAdd(left.exponent, right.exponent));
  }
  if (same(left, right)) return pow(left, { n: 2n, d: 1n });
  if (left.kind === 'pow' && same(left.base, right)) {
    return pow(left.base, rAdd(left.exponent, RAT_ONE));
  }
  if (right.kind === 'pow' && same(right.base, left)) {
    return pow(right.base, rAdd(right.exponent, RAT_ONE));
  }
  return { kind: 'mul', left, right };
}

function div(left: Expr, right: Expr): Expr {
  if (isZero(right)) throw new Error('Division by zero.');
  if (isZero(left)) return ZERO;
  if (isOne(right)) return left;
  if (same(left, right)) return ONE;
  if (left.kind === 'num' && right.kind === 'num') {
    return { kind: 'num', value: rDiv(left.value, right.value) };
  }
  if (right.kind === 'neg') return div(neg(left), right.child);
  if (left.kind === 'neg') return neg(div(left.child, right));
  if (left.kind === 'num' && right.kind === 'mul' && right.left.kind === 'num') {
    return mul(
      { kind: 'num', value: rDiv(left.value, right.left.value) },
      div(ONE, right.right)
    );
  }
  if (left.kind === 'div') return div(left.left, mul(left.right, right));
  if (right.kind === 'div') return div(mul(left, right.right), right.left);
  if (left.kind === 'mul') {
    const cancel = (factor: Expr, other: Expr): Expr | null => {
      if (same(factor, right)) return other;
      if (factor.kind === 'pow' && right.kind === 'pow' && same(factor.base, right.base)) {
        return mul(other, pow(factor.base, rSub(factor.exponent, right.exponent)));
      }
      if (factor.kind === 'pow' && same(factor.base, right)) {
        return mul(other, pow(factor.base, rSub(factor.exponent, RAT_ONE)));
      }
      if (right.kind === 'pow' && same(factor, right.base)) {
        return mul(other, pow(right.base, rSub(RAT_ONE, right.exponent)));
      }
      return null;
    };
    const leftCancel = cancel(left.left, left.right);
    if (leftCancel) return leftCancel;
    const rightCancel = cancel(left.right, left.left);
    if (rightCancel) return rightCancel;
  }
  if (left.kind === 'pow' && right.kind === 'pow' && same(left.base, right.base)) {
    return pow(left.base, rSub(left.exponent, right.exponent));
  }
  if (left.kind === 'pow' && same(left.base, right)) {
    return pow(left.base, rSub(left.exponent, RAT_ONE));
  }
  if (right.kind === 'pow' && same(right.base, left)) {
    return pow(right.base, rSub(RAT_ONE, right.exponent));
  }
  if (right.kind === 'num') {
    if (left.kind === 'mul' && left.left.kind === 'num') {
      return mul({ kind: 'num', value: rDiv(left.left.value, right.value) }, left.right);
    }
    return mul({ kind: 'num', value: rDiv(RAT_ONE, right.value) }, left);
  }
  return { kind: 'div', left, right };
}

function pow(base: Expr, exponent: Rational): Expr {
  if (exponent.n === 0n) return ONE;
  if (exponent.n === exponent.d) return base;
  if (isOne(base)) return ONE;
  if (isZero(base)) {
    if (exponent.n < 0n) throw new Error('Division by zero.');
    return ZERO;
  }
  if (base.kind === 'num') {
    if (exponent.d === 1n) {
      return { kind: 'num', value: rPow(base.value, exponent.n) };
    }
    if (exponent.n === 1n && exponent.d === 2n) {
      const root = exactSqrt(base.value);
      if (root) return { kind: 'num', value: root };
      if (base.value.n < 0n) {
        throw new Error('The square root of a negative number is not real.');
      }
      return { kind: 'pow', base, exponent };
    }
  }
  if (base.kind === 'pow') {
    return pow(base.base, rMul(base.exponent, exponent));
  }
  if (base.kind === 'mul' && exponent.d === 1n) {
    return mul(pow(base.left, exponent), pow(base.right, exponent));
  }
  if (base.kind === 'fn' && base.name === 'sqrt' && exponent.d === 1n && exponent.n % 2n === 0n) {
    return pow(base.arg, { n: exponent.n / 2n, d: 1n });
  }
  return { kind: 'pow', base, exponent };
}

function fn(name: FunctionName, arg: Expr): Expr {
  if (arg.kind === 'num') {
    const value = arg.value;
    if (name === 'sin' && value.n === 0n) return ZERO;
    if (name === 'cos' && value.n === 0n) return ONE;
    if (name === 'tan' && value.n === 0n) return ZERO;
    if (name === 'exp' && value.n === 0n) return ONE;
    if (name === 'sinh' && value.n === 0n) return ZERO;
    if (name === 'cosh' && value.n === 0n) return ONE;
    if (name === 'ln') {
      if (value.n <= 0n) throw new Error('The logarithm is only real for positive values.');
      if (value.n === value.d) return ZERO;
      return { kind: 'fn', name, arg };
    }
    if (name === 'sqrt') {
      if (value.n < 0n) throw new Error('The square root of a negative number is not real.');
      const root = exactSqrt(value);
      if (root) return { kind: 'num', value: root };
    }
  }
  return { kind: 'fn', name, arg };
}

function substitute(value: Expr, center: Rational): Expr {
  switch (value.kind) {
    case 'num':
    case 'const':
      return value;
    case 'var':
      return { kind: 'num', value: center };
    case 'add':
      return add(substitute(value.left, center), substitute(value.right, center));
    case 'sub':
      return sub(substitute(value.left, center), substitute(value.right, center));
    case 'mul':
      return mul(substitute(value.left, center), substitute(value.right, center));
    case 'div':
      return div(substitute(value.left, center), substitute(value.right, center));
    case 'neg':
      return neg(substitute(value.child, center));
    case 'pow':
      return pow(substitute(value.base, center), value.exponent);
    case 'fn':
      return fn(value.name, substitute(value.arg, center));
  }
}

function constantValue(value: Expr): Rational | null {
  switch (value.kind) {
    case 'num':
      return value.value;
    case 'neg': {
      const inner = constantValue(value.child);
      return inner ? rNeg(inner) : null;
    }
    case 'add': {
      const left = constantValue(value.left);
      const right = constantValue(value.right);
      return left && right ? rAdd(left, right) : null;
    }
    case 'sub': {
      const left = constantValue(value.left);
      const right = constantValue(value.right);
      return left && right ? rSub(left, right) : null;
    }
    case 'mul': {
      const left = constantValue(value.left);
      const right = constantValue(value.right);
      return left && right ? rMul(left, right) : null;
    }
    case 'div': {
      const left = constantValue(value.left);
      const right = constantValue(value.right);
      return left && right ? rDiv(left, right) : null;
    }
    case 'pow': {
      const base = constantValue(value.base);
      if (!base) return null;
      if (value.exponent.d === 1n) return rPow(base, value.exponent.n);
      if (value.exponent.n === 1n && value.exponent.d === 2n) return exactSqrt(base);
      return null;
    }
    default:
      return null;
  }
}

function derivative(value: Expr): Expr {
  switch (value.kind) {
    case 'num':
    case 'const':
      return ZERO;
    case 'var':
      return ONE;
    case 'add':
      return add(derivative(value.left), derivative(value.right));
    case 'sub':
      return sub(derivative(value.left), derivative(value.right));
    case 'mul':
      return add(mul(derivative(value.left), value.right), mul(value.left, derivative(value.right)));
    case 'div': {
      const numerator = sub(
        mul(derivative(value.left), value.right),
        mul(value.left, derivative(value.right))
      );
      return div(numerator, pow(value.right, { n: 2n, d: 1n }));
    }
    case 'neg':
      return neg(derivative(value.child));
    case 'pow': {
      const inner = derivative(value.base);
      const lowered = rSub(value.exponent, RAT_ONE);
      return mul(
        { kind: 'num', value: value.exponent },
        mul(pow(value.base, lowered), inner)
      );
    }
    case 'fn': {
      const inner = derivative(value.arg);
      switch (value.name) {
        case 'sin':
          return mul(fn('cos', value.arg), inner);
        case 'cos':
          return neg(mul(fn('sin', value.arg), inner));
        case 'tan':
          return div(inner, pow(fn('cos', value.arg), { n: 2n, d: 1n }));
        case 'exp':
          return mul(fn('exp', value.arg), inner);
        case 'ln':
          return div(inner, value.arg);
        case 'sqrt':
          return div(inner, mul(TWO, fn('sqrt', value.arg)));
        case 'sinh':
          return mul(fn('cosh', value.arg), inner);
        case 'cosh':
          return mul(fn('sinh', value.arg), inner);
      }
    }
  }
}

function ratLatex(value: Rational): string {
  if (value.d === 1n) return value.n.toString();
  const sign = value.n < 0n ? '-' : '';
  return `${sign}\\frac{${absBig(value.n).toString()}}{${value.d.toString()}}`;
}

function ratPlain(value: Rational): string {
  if (value.d === 1n) return value.n.toString();
  return `${value.n.toString()}/${value.d.toString()}`;
}

function wrapLatex(value: Expr, inner: string): string {
  return value.kind === 'add' || value.kind === 'sub' ? `\\left(${inner}\\right)` : inner;
}

function exprLatex(value: Expr): string {
  switch (value.kind) {
    case 'num':
      return ratLatex(value.value);
    case 'var':
      return 'x';
    case 'const':
      return value.name === 'pi' ? '\\pi' : 'e';
    case 'add':
      return `${exprLatex(value.left)} + ${exprLatex(value.right)}`;
    case 'sub':
      return `${exprLatex(value.left)} - ${exprLatex(value.right)}`;
    case 'mul': {
      if (value.left.kind === 'num' && value.right.kind !== 'num') {
        const coefficient = ratLatex(value.left.value);
        return `${coefficient}${wrapLatex(value.right, exprLatex(value.right))}`;
      }
      return `${wrapLatex(value.left, exprLatex(value.left))} \\cdot ${wrapLatex(
        value.right,
        exprLatex(value.right)
      )}`;
    }
    case 'div':
      return `\\frac{${exprLatex(value.left)}}{${exprLatex(value.right)}}`;
    case 'neg':
      return `-${wrapLatex(value.child, exprLatex(value.child))}`;
    case 'pow': {
      if (value.exponent.n === 1n && value.exponent.d === 2n) {
        return `\\sqrt{${exprLatex(value.base)}}`;
      }
      if (value.exponent.n === -1n && value.exponent.d === 1n) {
        return `\\frac{1}{${exprLatex(value.base)}}`;
      }
      const base =
        value.base.kind === 'add' ||
        value.base.kind === 'sub' ||
        value.base.kind === 'mul' ||
        value.base.kind === 'div' ||
        value.base.kind === 'pow'
          ? `\\left(${exprLatex(value.base)}\\right)`
          : exprLatex(value.base);
      return `${base}^{${ratLatex(value.exponent)}}`;
    }
    case 'fn':
      switch (value.name) {
        case 'exp':
          return `e^{${exprLatex(value.arg)}}`;
        case 'sqrt':
          return `\\sqrt{${exprLatex(value.arg)}}`;
        case 'ln':
          return `\\ln\\left(${exprLatex(value.arg)}\\right)`;
        default:
          return `\\${value.name}\\left(${exprLatex(value.arg)}\\right)`;
      }
  }
}

function wrapPlain(value: Expr, inner: string): string {
  return value.kind === 'add' ||
    value.kind === 'sub' ||
    value.kind === 'neg' ||
    value.kind === 'mul' ||
    value.kind === 'div'
    ? `(${inner})`
    : inner;
}

function exprPlain(value: Expr): string {
  switch (value.kind) {
    case 'num':
      return ratPlain(value.value);
    case 'var':
      return 'x';
    case 'const':
      return value.name;
    case 'add':
      return `${exprPlain(value.left)} + ${exprPlain(value.right)}`;
    case 'sub':
      return `${exprPlain(value.left)} - ${exprPlain(value.right)}`;
    case 'mul':
      return `${wrapPlain(value.left, exprPlain(value.left))}*${wrapPlain(
        value.right,
        exprPlain(value.right)
      )}`;
    case 'div':
      return `${wrapPlain(value.left, exprPlain(value.left))}/${wrapPlain(
        value.right,
        exprPlain(value.right)
      )}`;
    case 'neg':
      return `-${wrapPlain(value.child, exprPlain(value.child))}`;
    case 'pow':
      return `${wrapPlain(value.base, exprPlain(value.base))}^${ratPlain(value.exponent)}`;
    case 'fn':
      return `${value.name}(${exprPlain(value.arg)})`;
  }
}

function factorial(value: number): bigint {
  let result = 1n;
  for (let index = 2; index <= value; index += 1) result *= BigInt(index);
  return result;
}

function shiftLatex(center: Rational): string {
  if (center.n === 0n) return 'x';
  const magnitude = ratLatex({ n: absBig(center.n), d: center.d });
  return center.n > 0n ? `\\left(x - ${magnitude}\\right)` : `\\left(x + ${magnitude}\\right)`;
}

function shiftPlain(center: Rational): string {
  if (center.n === 0n) return 'x';
  const magnitude = ratPlain({ n: absBig(center.n), d: center.d });
  return center.n > 0n ? `(x - ${magnitude})` : `(x + ${magnitude})`;
}

function factorLatex(center: Rational, k: number): string {
  if (k === 0) return '';
  const shift = shiftLatex(center);
  return k === 1 ? shift : `${shift}^{${k}}`;
}

function termBodyLatex(coefficient: Expr, center: Rational, k: number): string {
  const factor = factorLatex(center, k);
  if (k === 0) return exprLatex(coefficient);
  if (isOne(coefficient)) return factor;
  if (coefficient.kind === 'num') return `${ratLatex(coefficient.value)}${factor}`;
  return `\\left(${exprLatex(coefficient)}\\right)${factor}`;
}

function termLatex(coefficient: Expr, center: Rational, k: number): string {
  if (isZero(coefficient)) return '0';
  if (isNeg(coefficient)) {
    return `-${termBodyLatex(neg(coefficient), center, k)}`;
  }
  return termBodyLatex(coefficient, center, k);
}

function seriesLatex(terms: { coefficient: Expr; k: number }[], center: Rational): string {
  const visible = terms.filter((term) => !isZero(term.coefficient));
  if (!visible.length) return '0';
  let output = '';
  visible.forEach((term, index) => {
    const negative = isNeg(term.coefficient);
    const coefficient = negative ? neg(term.coefficient) : term.coefficient;
    const body = termBodyLatex(coefficient, center, term.k);
    if (index === 0) output += negative ? `-${body}` : body;
    else output += negative ? ` - ${body}` : ` + ${body}`;
  });
  return output;
}

function seriesPlain(terms: { coefficient: Expr; k: number }[], center: Rational): string {
  const visible = terms.filter((term) => !isZero(term.coefficient));
  if (!visible.length) return '0';
  let output = '';
  visible.forEach((term, index) => {
    const negative = isNeg(term.coefficient);
    const coefficient = negative ? neg(term.coefficient) : term.coefficient;
    const factor = term.k === 0 ? '' : term.k === 1 ? shiftPlain(center) : `${shiftPlain(center)}^${term.k}`;
    const body =
      term.k === 0
        ? exprPlain(coefficient)
        : isOne(coefficient)
          ? factor
          : coefficient.kind === 'num'
            ? `${ratPlain(coefficient.value)}${factor}`
            : `(${exprPlain(coefficient)})${factor}`;
    if (index === 0) output += negative ? `-${body}` : body;
    else output += negative ? ` - ${body}` : ` + ${body}`;
  });
  return output;
}

class Parser {
  private index = 0;

  constructor(private readonly source: string) {}

  parse(): Expr {
    const value = this.parseSum();
    this.skip();
    if (this.index < this.source.length) {
      throw new Error(`Unexpected “${this.source[this.index]}”.`);
    }
    return value;
  }

  private skip(): void {
    while (/\s/.test(this.source[this.index] ?? '')) this.index += 1;
  }

  private peek(): string {
    this.skip();
    return this.source[this.index] ?? '';
  }

  private take(): string {
    this.skip();
    return this.source[this.index++] ?? '';
  }

  private expect(character: string): void {
    if (this.take() !== character) throw new Error(`Expected “${character}”.`);
  }

  private parseSum(): Expr {
    let value = this.parseTerm();
    while (true) {
      const next = this.peek();
      if (next === '+') {
        this.take();
        value = add(value, this.parseTerm());
      } else if (next === '-') {
        this.take();
        value = sub(value, this.parseTerm());
      } else {
        return value;
      }
    }
  }

  private parseTerm(): Expr {
    let value = this.parseUnary();
    while (true) {
      const next = this.peek();
      if (next === '*') {
        this.take();
        value = mul(value, this.parseUnary());
      } else if (next === '/') {
        this.take();
        value = div(value, this.parseUnary());
      } else if (this.startsPrimary()) {
        value = mul(value, this.parseUnary());
      } else {
        return value;
      }
    }
  }

  private parseUnary(): Expr {
    const next = this.peek();
    if (next === '-') {
      this.take();
      return neg(this.parseUnary());
    }
    if (next === '+') {
      this.take();
      return this.parseUnary();
    }
    return this.parsePower();
  }

  private parsePower(): Expr {
    const base = this.parsePrimary();
    if (this.peek() === '^') {
      this.take();
      const exponentExpr = this.parseUnary();
      const exponent = constantValue(exponentExpr);
      if (exponent) return pow(base, exponent);
      if (base.kind === 'const' && base.name === 'e') return fn('exp', exponentExpr);
      throw new Error('Only constant exponents are supported; use exp(...) for exponentials.');
    }
    return base;
  }

  private startsPrimary(): boolean {
    const next = this.peek();
    return next !== '' && (/[0-9.]/.test(next) || /[a-zA-Z]/.test(next) || next === '(');
  }

  private parsePrimary(): Expr {
    const next = this.peek();
    if (next === '(') {
      this.take();
      const value = this.parseSum();
      this.expect(')');
      return value;
    }
    if (/[0-9.]/.test(next)) return this.parseNumber();
    if (/[a-zA-Z]/.test(next)) return this.parseIdentifier();
    throw new Error(next ? `Unexpected “${next}”.` : 'The expression ends early.');
  }

  private parseNumber(): Expr {
    const match = /^(?:\d+(?:\.\d+)?|\.\d+)/.exec(this.source.slice(this.index));
    if (!match) throw new Error('Expected a number.');
    this.index += match[0].length;
    if (match[0].includes('.')) {
      const [whole, fraction = ''] = match[0].split('.');
      return numValue(BigInt(`${whole || '0'}${fraction}`), 10n ** BigInt(fraction.length));
    }
    return numValue(BigInt(match[0]));
  }

  private parseIdentifier(): Expr {
    const match = /^[a-zA-Z]+/.exec(this.source.slice(this.index));
    if (!match) throw new Error('Expected a name.');
    this.index += match[0].length;
    const name = match[0].toLowerCase();
    if (name === 'x') return { kind: 'var' };
    if (name === 'pi') return { kind: 'const', name: 'pi' };
    if (name === 'e') return { kind: 'const', name: 'e' };
    if (
      name === 'sin' ||
      name === 'cos' ||
      name === 'tan' ||
      name === 'exp' ||
      name === 'ln' ||
      name === 'sqrt' ||
      name === 'sinh' ||
      name === 'cosh'
    ) {
      let argument: Expr;
      if (this.peek() === '(') {
        this.take();
        argument = this.parseSum();
        this.expect(')');
      } else {
        argument = this.parsePower();
      }
      if (name === 'sqrt') return pow(argument, RAT_HALF);
      return fn(name, argument);
    }
    throw new Error(`Unknown name “${match[0]}”. Use x, pi, e, sin, cos, tan, exp, ln, sqrt, sinh or cosh.`);
  }
}

function normalizeSource(source: string): string {
  return source
    .replace(/[×·]/g, '*')
    .replace(/[÷]/g, '/')
    .replace(/[−–]/g, '-')
    .replace(/π/g, 'pi')
    .replace(/√/g, 'sqrt ');
}

export function parseExpression(source: string): Expr {
  const text = normalizeSource(source).trim();
  if (!text) throw new Error('Enter a function of x.');
  return new Parser(text).parse();
}

export function expandSeries(source: string, centerText: string, order: number): SeriesResult {
  if (!Number.isInteger(order) || order < 0 || order > MAX_ORDER) {
    throw new Error(`Choose a whole number of terms between 0 and ${MAX_ORDER}.`);
  }
  const tree = parseExpression(source);
  const center = constantValue(parseExpression(centerText));
  if (!center) throw new Error('The centre must be a rational number, for example 0, 1 or 1/2.');

  const kind = center.n === 0n ? 'maclaurin' : 'taylor';
  const centerPlain = ratPlain(center);
  const centerLatex = ratLatex(center);
  const steps: SeriesStep[] = [];
  const terms: { coefficient: Expr; k: number }[] = [];
  let current = tree;

  for (let k = 0; k <= order; k += 1) {
    const value = substitute(current, center);
    const coefficient = div(value, { kind: 'num', value: { n: factorial(k), d: 1n } });
    terms.push({ coefficient, k });
    steps.push({
      k,
      derivativeLatex: exprLatex(current),
      valueLatex: exprLatex(value),
      coefficientLatex: exprLatex(coefficient),
      coefficientPlain: exprPlain(coefficient),
      termLatex: termLatex(coefficient, center, k)
    });
    if (k < order) current = derivative(current);
  }

  const label = kind === 'maclaurin' ? 'Maclaurin' : 'Taylor';
  const series = seriesLatex(terms, center);
  const seriesPlainText = seriesPlain(terms, center);
  const rows = steps
    .map(
      (step) =>
        `${step.k} & ${step.derivativeLatex} & ${step.valueLatex} & ${step.coefficientLatex}`
    )
    .join(' \\\\\n');

  const latex = latexDocument({
    title: `${label} series`,
    sections: [
      {
        title: 'Function and centre',
        body: `\\[\nf(x) = ${exprLatex(tree)}\n\\]\nThe ${label} expansion about $x = ${centerLatex}$ to degree ${order}.`
      },
      {
        title: 'Derivatives and coefficients',
        body: `\\[\n\\begin{array}{c|l|l|l}\nk & f^{(k)}(x) & f^{(k)}(a) & c_k \\\\\n\\hline\n${rows}\n\\end{array}\n\\]`
      },
      {
        title: 'Series',
        body: `\\[\nT_{${order}}(x) = ${series}\n\\]`
      }
    ]
  });

  return {
    source: source.trim(),
    functionLatex: exprLatex(tree),
    center: centerPlain,
    centerLatex,
    order,
    kind,
    seriesLatex: series,
    steps,
    summary: [
      `${label} series of f(x) = ${exprPlain(tree)} about x = ${centerPlain}, degree ${order}.`,
      `Coefficients use c_k = f^{(k)}(a)/k!, with a = ${centerPlain}.`,
      `The expanded polynomial is ${seriesPlainText}.`
    ],
    summaryLatex: [
      `f(x) = ${exprLatex(tree)}`,
      `a = ${centerLatex}`,
      `T_{${order}}(x) = ${series}`
    ],
    latex
  };
}
