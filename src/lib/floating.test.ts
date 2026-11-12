import { describe, expect, it } from 'vitest';
import {
  convertBases,
  decodeIeee,
  encodeIeee,
  normalise,
  parseRational
} from './floating';

describe('parseRational', () => {
  it('parses integers and fractions in a base', () => {
    expect(parseRational('FF', 16)).toEqual({ num: 255n, den: 1n });
    expect(parseRational('10.1', 2)).toEqual({ num: 5n, den: 2n });
  });

  it('rejects invalid digits', () => {
    expect(() => parseRational('2', 2)).toThrow(/not valid in base 2/);
  });
});

describe('convertBases', () => {
  it('writes the same value in all four bases', () => {
    const result = convertBases('255', 10);
    const byBase = Object.fromEntries(
      result.outputs.map((output) => [output.base, output.text])
    );
    expect(byBase[2]).toBe('11111111');
    expect(byBase[8]).toBe('377');
    expect(byBase[10]).toBe('255');
    expect(byBase[16]).toBe('FF');
  });
});

describe('normalise', () => {
  it('normalises a binary fraction', () => {
    const result = normalise('101.1', 2);
    expect(result.exponent).toBe(2);
    expect(result.fraction).toBe('011');
    expect(result.exact).toBe(true);
  });
});

describe('IEEE 754', () => {
  it('encodes one as a single-precision float', () => {
    const result = encodeIeee('1', 'single');
    expect(result.hex).toBe('3F800000');
    expect(result.classification).toBe('Normal');
    expect(result.bitString).toHaveLength(32);
  });

  it('decodes the same value', () => {
    const result = decodeIeee('3F800000', 'single');
    expect(result.value).toBe('1');
    expect(result.classification).toBe('Normal');
  });

  it('rejects non-finite input', () => {
    expect(() => encodeIeee('Infinity', 'single')).toThrow(/finite/);
  });
});
