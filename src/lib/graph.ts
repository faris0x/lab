export type GraphAlgorithm = 'dijkstra' | 'floyd' | 'kruskal' | 'prim';

export interface Graph {
  labels: string[];
  directed: boolean;
  weights: string[][];
}

export interface GraphStep {
  text: string;
  latex?: string;
}

export interface GraphResult {
  title: string;
  summary: string[];
  steps: GraphStep[];
  latex: string;
}

interface GraphError {
  ok: false;
  error: string;
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
      const response = await fetch('/wasm/graphnet.wasm');
      if (!response.ok) {
        throw new Error('The graph module could not be loaded.');
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

export function preloadGraph(): void {
  void load().catch(() => {});
}

export async function runGraph(
  graph: Graph,
  algorithm: GraphAlgorithm,
  source = 0,
  target = graph.labels.length - 1
): Promise<GraphResult> {
  const wasm = await load();
  const payload = {
    labels: graph.labels,
    directed: graph.directed,
    weights: graph.weights,
    algorithm,
    source,
    target
  };
  const bytes = new TextEncoder().encode(JSON.stringify(payload));
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

  const output = JSON.parse(json) as (GraphResult & { ok: true }) | GraphError;
  if (!('ok' in output) || !output.ok) {
    throw new Error('error' in output ? output.error : 'The graph could not be solved.');
  }
  return output;
}
