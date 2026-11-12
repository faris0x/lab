import { preloadSolver, solveJson } from './wasm';

export type ComplexOperation = 'arithmetic' | 'polar' | 'power' | 'roots';

export interface ComplexStep {
  description: string;
  descriptionLatex: string;
}

export interface ArgandPoint {
  label: string;
  labelLatex: string;
  x: number;
  y: number;
  kind: 'input' | 'result' | 'root';
}

export interface ArgandCircle {
  radius: number;
}

export interface ComplexOutput {
  ok: true;
  operation: ComplexOperation;
  zLatex: string;
  wLatex: string | null;
  steps: ComplexStep[];
  summary: string[];
  summaryLatex: string[];
  lines: string[];
  latex: { document: string; blocks: string[] };
  argand: { points: ArgandPoint[]; circles: ArgandCircle[] };
}

interface ComplexError {
  ok: false;
  error: string;
}

export interface ComplexInput {
  operation: ComplexOperation;
  z: [string, string];
  w?: [string, string];
  op?: '+' | '-' | '*' | '/';
  n?: number;
}

export function preloadComplex(): void {
  preloadSolver('complexnum');
}

export async function solveComplex(input: ComplexInput): Promise<ComplexOutput> {
  const output = await solveJson<ComplexOutput | ComplexError>(
    'complexnum',
    input
  );
  if (!output.ok) {
    throw new Error(output.error);
  }
  return output;
}
