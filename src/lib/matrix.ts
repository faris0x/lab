import { preloadSolver, solveJson } from './wasm';

export type MatrixOperation =
  | 'rref'
  | 'det'
  | 'inverse'
  | 'nullspace'
  | 'solve';

export interface MatrixStep {
  kind: string;
  matrixLatex: string;
  description: string;
  descriptionLatex: string;
  pivot: [number, number] | null;
}

export interface MatrixResult {
  matrix: {
    label: string;
    labelLatex: string;
    latex: string;
  } | null;
  scalar: {
    label: string;
    labelLatex: string;
    plain: string;
    latex: string;
  } | null;
  vectors: {
    label: string;
    labelLatex: string;
    latex: string;
  }[];
  text: string[];
  textLatex: string[];
}

export interface MatrixOutput {
  ok: true;
  operation: MatrixOperation;
  inputLatex: string;
  steps: MatrixStep[];
  result: MatrixResult;
  summary: string[];
  summaryLatex: string[];
  latex: { document: string; blocks: string[] };
}

interface MatrixError {
  ok: false;
  error: string;
}

export interface MatrixInput {
  operation: MatrixOperation;
  matrix: string[][];
  rhs?: string[];
}

export function preloadMatrix(): void {
  preloadSolver('linalg');
}

export async function solveMatrix(input: MatrixInput): Promise<MatrixOutput> {
  const output = await solveJson<MatrixOutput | MatrixError>('linalg', input);
  if (!output.ok) {
    throw new Error(output.error);
  }
  return output;
}
