import { describe, expect, it } from 'vitest';
import { simplifyBoolean } from './boolean';

describe('simplifyBoolean', () => {
  it('simplifies a redundant expression to a single literal', () => {
    const result = simplifyBoolean('a & b | a & ~b');
    expect(result.variables).toEqual(['a', 'b']);
    expect(result.minimal).toBe('a');
  });

  it('returns constants unchanged', () => {
    expect(simplifyBoolean('a').minimal).toBe('a');
    expect(simplifyBoolean('a | ~a').minimal).toBe('1');
    expect(simplifyBoolean('a & ~a').minimal).toBe('0');
  });

  it('lists minterms in truth-table order', () => {
    const result = simplifyBoolean('a & b');
    expect(result.minterms).toEqual([3]);
    expect(result.maxterms).toEqual([0, 1, 2]);
  });

  it('rejects too many variables', () => {
    expect(() => simplifyBoolean('a|b|c|d|e|g|h')).toThrow(/at most six/);
  });
});
