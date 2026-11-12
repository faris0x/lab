import { preloadSolver, solveJson } from './wasm';

export interface Rat {
  n: number;
  d: number;
}

export interface Cell {
  a: Rat;
  b: Rat;
}

export interface Ratio {
  row: number;
  value: Rat;
  plain: string;
  latex: string;
}

export interface RowOp {
  target: number;
  source: number;
  factor: Rat;
  plain: string;
  latex: string;
}

export type StepKind = 'iteration' | 'optimal' | 'unbounded' | 'infeasible';

export interface Step {
  kind: StepKind;
  index: number;
  headers: string[];
  headersLatex: string[];
  rows: Cell[][];
  rhs: Cell[];
  obj: Cell[];
  objRhs: Cell;
  basis: string[];
  pivot: { row: number; col: number } | null;
  entering: number | null;
  leaving: number | null;
  ratios: Ratio[];
  rowOps: RowOp[];
  prose: string[];
  proseLatex: string[];
  latex: string;
}

export interface SolutionValue {
  var: string;
  value: Rat;
  plain: string;
  latex: string;
}

export interface StandardForm {
  objective: string;
  objectiveLatex: string;
  constraints: string[];
  constraintsLatex: string[];
  objectiveRow: string;
  objectiveRowLatex: string;
}

export interface SolveOutput {
  ok: true;
  status: 'optimal' | 'unbounded' | 'infeasible';
  sense: 'max' | 'min';
  transformed: boolean;
  vars: string[];
  objectiveLabel: string;
  objective: string;
  objectiveLatex: string;
  standardForm: StandardForm;
  steps: Step[];
  solution: SolutionValue[];
  optimum: { plain: string; latex: string } | null;
  summary: string[];
  summaryLatex: string[];
  latex: { document: string; blocks: string[] };
  iterations: number;
}

interface SolveError {
  ok: false;
  error: string;
}

export interface SolveInput {
  sense: 'max' | 'min';
  vars: string[];
  objective: string[];
  constraints: { coeffs: string[]; op: '<=' | '>=' | '='; rhs: string }[];
}

/** Warm the WASM module without blocking the first paint. */
export function preload(): void {
  preloadSolver('simplex');
}

export async function solve(input: SolveInput): Promise<SolveOutput> {
  const output = await solveJson<SolveOutput | SolveError>('simplex', input);
  if (!output.ok) {
    throw new Error(output.error);
  }
  return output;
}
