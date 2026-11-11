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
      const response = await fetch('/wasm/linalg.wasm');
      if (!response.ok) {
        throw new Error('The matrix module could not be loaded.');
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

export function preloadMatrix(): void {
  void load().catch(() => {});
}

export async function solveMatrix(input: MatrixInput): Promise<MatrixOutput> {
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

  const output = JSON.parse(json) as MatrixOutput | MatrixError;
  if (!output.ok) {
    throw new Error(output.error);
  }
  return output;
}
