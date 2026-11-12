import { describe, expect, it } from 'vitest';
import { expandSeries, parseExpression } from './series';

function coefficients(source: string, center: string, order: number): string[] {
  return expandSeries(source, center, order).steps.map((step) => step.coefficientPlain);
}

describe('series parser', () => {
  it('supports implicit multiplication', () => {
    expect(coefficients('2x', '0', 1)).toEqual(['0', '2']);
    expect(coefficients('x(x+1)', '0', 2)).toEqual(['0', '1', '1']);
  });

  it('rejects unknown names', () => {
    expect(() => parseExpression('sinh(x) + y')).toThrow(/Unknown name/);
  });
});

describe('Maclaurin series', () => {
  it('expands exp(x)', () => {
    const result = expandSeries('exp(x)', '0', 5);
    expect(result.kind).toBe('maclaurin');
    expect(coefficients('exp(x)', '0', 5)).toEqual([
      '1',
      '1',
      '1/2',
      '1/6',
      '1/24',
      '1/120'
    ]);
    expect(result.seriesLatex).toContain('x^{5}');
  });

  it('expands sin(x) with alternating signs', () => {
    expect(coefficients('sin(x)', '0', 7)).toEqual([
      '0',
      '1',
      '0',
      '-1/6',
      '0',
      '1/120',
      '0',
      '-1/5040'
    ]);
  });

  it('expands cos(x)', () => {
    expect(coefficients('cos(x)', '0', 4)).toEqual(['1', '0', '-1/2', '0', '1/24']);
  });

  it('expands ln(1+x)', () => {
    expect(coefficients('ln(1+x)', '0', 4)).toEqual(['0', '1', '-1/2', '1/3', '-1/4']);
  });

  it('expands 1/(1-x)', () => {
    expect(coefficients('1/(1-x)', '0', 4)).toEqual(['1', '1', '1', '1', '1']);
  });

  it('expands ln(1-3x) with implicit multiplication', () => {
    expect(coefficients('ln(1-3x)', '0', 3)).toEqual(['0', '-3', '-9/2', '-9']);
  });

  it('handles e^x notation', () => {
    expect(coefficients('e^x', '0', 3)).toEqual(['1', '1', '1/2', '1/6']);
  });
});

describe('Taylor series', () => {
  it('expands a polynomial about a non-zero centre', () => {
    expect(coefficients('x^2', '1', 3)).toEqual(['1', '2', '1', '0']);
  });

  it('expands sqrt(x) about 1', () => {
    expect(coefficients('sqrt(x)', '1', 3)).toEqual(['1', '1/2', '-1/8', '1/16']);
  });

  it('keeps transcendental values symbolic at a non-zero centre', () => {
    expect(coefficients('sin(x)', '1', 2)).toEqual(['sin(1)', 'cos(1)', '-1/2*sin(1)']);
  });

  it('expands a fractional power', () => {
    expect(coefficients('(1+x)^(1/2)', '0', 3)).toEqual(['1', '1/2', '-1/8', '1/16']);
  });
});

describe('errors', () => {
  it('rejects invalid logarithms', () => {
    expect(() => expandSeries('ln(x)', '0', 3)).toThrow(/positive/);
  });

  it('rejects negative square roots', () => {
    expect(() => expandSeries('sqrt(x)', '-1', 3)).toThrow(/negative/);
  });

  it('rejects an order that is too large', () => {
    expect(() => expandSeries('sin(x)', '0', 13)).toThrow(/between 0 and 12/);
  });

  it('rejects a non-rational centre', () => {
    expect(() => expandSeries('sin(x)', 'pi', 3)).toThrow(/rational/);
  });
});
