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

interface Exports {
  memory: WebAssembly.Memory;
  alloc(length: number): number;
  dealloc(ptr: number, length: number): void;
  solve(ptr: number, length: number): number;
}

let instancePromise: Promise<Exports> | null = null;

function load(): Promise<Exports> {
  if (!instancePromise) {
    instancePromise = (async () => {
      const response = await fetch('/wasm/simplex.wasm');
      if (!response.ok) {
        throw new Error('The solver module could not be loaded.');
      }
      let instance: WebAssembly.Instance;
      try {
        ({ instance } = await WebAssembly.instantiateStreaming(response, {}));
      } catch {
        const bytes = await response.arrayBuffer();
        ({ instance } = await WebAssembly.instantiate(bytes, {}));
      }
      return instance.exports as unknown as Exports;
    })().catch((error) => {
      instancePromise = null;
      throw error;
    });
  }
  return instancePromise;
}

/** Warm the WASM module without blocking the first paint. */
export function preload(): void {
  void load().catch(() => {});
}

export async function solve(input: SolveInput): Promise<SolveOutput> {
  const wasm = await load();
  const bytes = new TextEncoder().encode(JSON.stringify(input));
  const ptr = wasm.alloc(bytes.byteLength);
  new Uint8Array(wasm.memory.buffer, ptr, bytes.byteLength).set(bytes);
  const outPtr = wasm.solve(ptr, bytes.byteLength);
  wasm.dealloc(ptr, bytes.byteLength);

  const view = new DataView(wasm.memory.buffer);
  const length = view.getUint32(outPtr, true);
  const json = new TextDecoder().decode(
    new Uint8Array(wasm.memory.buffer, outPtr + 4, length)
  );
  wasm.dealloc(outPtr, 4 + length);

  const output = JSON.parse(json) as SolveOutput | SolveError;
  if (!output.ok) {
    throw new Error(output.error);
  }
  return output;
}
