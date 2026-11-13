import { preloadSolver, solveJson } from './wasm';

export interface BooleanRow {
  values: boolean[];
}

export interface PrimeImplicant {
  bits: string;
  expression: string;
  covers: number[];
}

export interface BooleanResult {
  formula: string;
  variables: string[];
  rows: BooleanRow[];
  minterms: number[];
  maxterms: number[];
  canonicalSop: string;
  canonicalPos: string;
  primes: PrimeImplicant[];
  essential: PrimeImplicant[];
  selected: PrimeImplicant[];
  minimal: string;
  latex: string;
}

interface BooleanError {
  ok: false;
  error: string;
}

interface BooleanSuccess {
  ok: true;
  result: BooleanResult;
}

export function preloadBoolean(): void {
  preloadSolver('boolean');
}

export async function simplifyBoolean(source: string): Promise<BooleanResult> {
  const output = await solveJson<BooleanSuccess | BooleanError>('boolean', {
    formula: source
  });
  if (!output.ok) {
    throw new Error(output.error);
  }
  return output.result;
}
