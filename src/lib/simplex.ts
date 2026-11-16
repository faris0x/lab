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

export type StepKind = 'start' | 'iteration' | 'optimal' | 'unbounded' | 'infeasible';

export interface Step {
  kind: StepKind;
  phase: number;
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
  mode: 'model' | 'tableau';
  method: 'bigm' | 'two-stage' | 'tableau';
  status: 'optimal' | 'unbounded' | 'infeasible';
  sense: 'max' | 'min';
  transformed: boolean;
  vars: string[];
  objectiveLabel: string;
  objective: string;
  objectiveLatex: string;
  standardForm: StandardForm | null;
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
  method?: 'bigm' | 'two-stage';
  vars: string[];
  objective: string[];
  constraints: { coeffs: string[]; op: '<=' | '>=' | '='; rhs: string }[];
}

/** Warm the WASM module without blocking the first paint. */
export function preload(): void {
  preloadSolver('simplex');
}

export interface FinishTableauInput {
  objectiveLabel: string;
  reportAs: 'max' | 'min';
  headers: string[];
  basis: number[];
  decisionColumns: number[];
  artificialColumns: number[];
  rows: string[][];
  rhs: string[];
  obj: string[];
  objRhs: string;
}

async function runSimplex(input: Record<string, unknown>): Promise<SolveOutput> {
  const output = await solveJson<SolveOutput | SolveError>('simplex', input);
  if (!output.ok) {
    throw new Error(output.error);
  }
  return output;
}

export function solve(input: SolveInput): Promise<SolveOutput> {
  return runSimplex({ mode: 'model', ...input });
}

export function finishTableau(input: FinishTableauInput): Promise<SolveOutput> {
  return runSimplex({ mode: 'tableau', ...input });
}
