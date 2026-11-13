import { preloadSolver, solveJson } from './wasm';

export interface TableColumn {
  label: string;
  key: string;
}

export interface TableRow {
  values: boolean[];
}

export type Verdict = 'tautology' | 'contradiction' | 'contingent';

export interface LogicResult {
  formula: string;
  variables: string[];
  columns: TableColumn[];
  rows: TableRow[];
  verdict: Verdict;
  latex: string;
}

interface LogicError {
  ok: false;
  error: string;
}

interface LogicSuccess {
  ok: true;
  result: LogicResult;
}

export function preloadLogic(): void {
  preloadSolver('logic');
}

export async function build(formula: string): Promise<LogicResult> {
  const output = await solveJson<LogicSuccess | LogicError>('logic', {
    formula
  });
  if (!output.ok) {
    throw new Error(output.error);
  }
  return output.result;
}
