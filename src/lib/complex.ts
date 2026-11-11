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
      const response = await fetch('/wasm/complexnum.wasm');
      if (!response.ok) {
        throw new Error('The complex-number module could not be loaded.');
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

export function preloadComplex(): void {
  void load().catch(() => {});
}

export async function solveComplex(input: ComplexInput): Promise<ComplexOutput> {
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

  const output = JSON.parse(json) as ComplexOutput | ComplexError;
  if (!output.ok) {
    throw new Error(output.error);
  }
  return output;
}
