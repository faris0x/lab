import { describe, expect, it } from 'vitest';
import { build, evaluate, parse } from './logic';

describe('parse', () => {
  it('normalises Unicode operators', () => {
    const tree = parse('p ∧ ¬q');
    expect(
      evaluate(tree, { p: true, q: false })
    ).toBe(true);
  });

  it('rejects unknown symbols', () => {
    expect(() => parse('p ? q')).toThrow(/Unknown symbol/);
  });
});

describe('build', () => {
  it('detects a tautology', () => {
    const result = build('p | ~p');
    expect(result.variables).toEqual(['p']);
    expect(result.rows).toHaveLength(2);
    expect(result.verdict).toBe('tautology');
  });

  it('detects a contradiction', () => {
    expect(build('p & ~p').verdict).toBe('contradiction');
  });

  it('classifies an implication as contingent', () => {
    const result = build('p -> q');
    expect(result.variables).toEqual(['p', 'q']);
    expect(result.rows).toHaveLength(4);
    expect(result.verdict).toBe('contingent');
  });

  it('handles constants', () => {
    expect(build('⊤').verdict).toBe('tautology');
    expect(build('⊥').verdict).toBe('contradiction');
  });
});
