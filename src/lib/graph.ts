import { preloadSolver, solveJson } from './wasm';

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

export function preloadGraph(): void {
  preloadSolver('graphnet');
}

export async function runGraph(
  graph: Graph,
  algorithm: GraphAlgorithm,
  source = 0,
  target = graph.labels.length - 1
): Promise<GraphResult> {
  const payload = {
    labels: graph.labels,
    directed: graph.directed,
    weights: graph.weights,
    algorithm,
    source,
    target
  };
  const output = await solveJson<(GraphResult & { ok: true }) | GraphError>(
    'graphnet',
    payload
  );
  if (!('ok' in output) || !output.ok) {
    throw new Error(
      'error' in output ? output.error : 'The graph could not be solved.'
    );
  }
  return output;
}
