export type GraphAlgorithm = 'dijkstra' | 'floyd' | 'kruskal' | 'prim';

export interface Graph {
  labels: string[];
  directed: boolean;
  weights: (number | null)[][];
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

interface Edge {
  u: number;
  v: number;
  w: number;
}

function edgesOf(graph: Graph): Edge[] {
  const edges: Edge[] = [];
  for (let i = 0; i < graph.labels.length; i += 1) {
    const start = graph.directed ? 0 : i + 1;
    for (let j = start; j < graph.labels.length; j += 1) {
      if (i === j) continue;
      const forward = graph.weights[i]?.[j] ?? null;
      const reverse = !graph.directed ? graph.weights[j]?.[i] ?? null : null;
      const weight = forward ?? reverse;
      if (weight !== null && weight > 0) edges.push({ u: i, v: j, w: weight });
    }
  }
  return edges;
}

function labelOf(graph: Graph, index: number): string {
  return graph.labels[index] ?? String(index);
}

function matrixLatex(graph: Graph, rows: string[][]): string {
  const header = ['', ...graph.labels].map((label) => (label ? `\\text{${label}}` : '')).join(' & ');
  const body = rows
    .map((row, index) => [`\\text{${graph.labels[index]}}`, ...row].join(' & '))
    .join(' \\\\\n');
  return `\\begin{array}{c|${'c'.repeat(graph.labels.length)}}
${header} \\\\ \\hline
${body}
\\end{array}`;
}

function numberText(value: number): string {
  return Number.isFinite(value) ? String(value) : '\\infty';
}

export function runGraph(
  graph: Graph,
  algorithm: GraphAlgorithm,
  source = 0,
  target = graph.labels.length - 1
): GraphResult {
  const n = graph.labels.length;
  if (n < 2) throw new Error('Add at least two nodes.');
  if (n > 9) throw new Error('Use at most nine nodes.');
  const edges = edgesOf(graph);

  if (algorithm === 'dijkstra') return dijkstra(graph, edges, source, target);
  if (algorithm === 'floyd') return floyd(graph);
  if (algorithm === 'kruskal') return kruskal(graph, edges);
  return prim(graph, edges, source);
}

function dijkstra(graph: Graph, edges: Edge[], source: number, target: number): GraphResult {
  if (edges.some((edge) => edge.w < 0)) throw new Error('Dijkstra needs non-negative weights.');
  const n = graph.labels.length;
  const dist = Array(n).fill(Infinity);
  const previous = Array(n).fill(-1);
  const visited = Array(n).fill(false);
  dist[source] = 0;
  const steps: GraphStep[] = [];

  for (let count = 0; count < n; count += 1) {
    let current = -1;
    for (let i = 0; i < n; i += 1) {
      if (!visited[i] && (current === -1 || dist[i] < dist[current])) current = i;
    }
    if (current === -1 || !Number.isFinite(dist[current])) break;
    visited[current] = true;
    steps.push({
      text: `Visit $${labelOf(graph, current)}$ with tentative distance $${numberText(dist[current])}$.`
    });

    for (let next = 0; next < n; next += 1) {
      const w = graph.weights[current]?.[next] ?? null;
      const reverse = !graph.directed ? graph.weights[next]?.[current] ?? null : null;
      const weight = w ?? reverse;
      if (weight === null || weight <= 0 || visited[next]) continue;
      const candidate = dist[current] + weight;
      if (candidate < dist[next]) {
        dist[next] = candidate;
        previous[next] = current;
        steps.push({
          text: `Relax $${labelOf(graph, current)} \\to ${labelOf(graph, next)}$: $${numberText(dist[current])} + ${weight} = ${candidate}$, so $d(${labelOf(graph, next)}) = ${candidate}$.`
        });
      }
    }
  }

  const path: number[] = [];
  let cursor = target;
  while (cursor !== -1) {
    path.unshift(cursor);
    if (cursor === source) break;
    cursor = previous[cursor];
  }
  const reachable = Number.isFinite(dist[target]);
  const pathText = reachable ? path.map((index) => labelOf(graph, index)).join(' → ') : 'no path';
  const summary = reachable
    ? [`Shortest distance ${labelOf(graph, source)} → ${labelOf(graph, target)}: ${dist[target]}.`, `Path: ${pathText}.`]
    : [`There is no path from ${labelOf(graph, source)} to ${labelOf(graph, target)}.`];

  return {
    title: 'Dijkstra shortest path',
    summary,
    steps,
    latex: documentLatex('Dijkstra shortest path', graph, summary, steps, edges)
  };
}

function floyd(graph: Graph): GraphResult {
  const n = graph.labels.length;
  const dist: number[][] = Array.from({ length: n }, (_, i) =>
    Array.from({ length: n }, (_, j) => {
      if (i === j) return 0;
      const forward = graph.weights[i]?.[j] ?? null;
      const reverse = !graph.directed ? graph.weights[j]?.[i] ?? null : null;
      const weight = forward ?? reverse;
      return weight === null || weight <= 0 ? Infinity : weight;
    })
  );
  const steps: GraphStep[] = [
    {
      text: 'Initialise the distance matrix; missing edges are $\\infty$ and the diagonal is $0$.',
      latex: matrixLatex(graph, dist.map((row) => row.map(numberText)))
    }
  ];

  for (let k = 0; k < n; k += 1) {
    for (let i = 0; i < n; i += 1) {
      for (let j = 0; j < n; j += 1) {
        if (dist[i][k] + dist[k][j] < dist[i][j]) {
          dist[i][j] = dist[i][k] + dist[k][j];
        }
      }
    }
    steps.push({
      text: `Allow paths through $${labelOf(graph, k)}$; update every entry using $d(i,j) = \\min(d(i,j), d(i,${labelOf(graph, k)}) + d(${labelOf(graph, k)},j))$.`,
      latex: matrixLatex(graph, dist.map((row) => row.map(numberText)))
    });
  }

  const summary = dist.map(
    (row, i) =>
      `${labelOf(graph, i)}: ${row.map((value, j) => `${labelOf(graph, j)}=${numberText(value)}`).join(', ')}`
  );
  return {
    title: 'Floyd–Warshall all-pairs shortest paths',
    summary,
    steps,
    latex: documentLatex('Floyd–Warshall all-pairs shortest paths', graph, summary, steps, edgesOf(graph))
  };
}

class DSU {
  parent: number[];
  rank: number[];
  constructor(n: number) {
    this.parent = Array.from({ length: n }, (_, i) => i);
    this.rank = Array(n).fill(0);
  }
  find(x: number): number {
    if (this.parent[x] !== x) this.parent[x] = this.find(this.parent[x]);
    return this.parent[x];
  }
  union(a: number, b: number): boolean {
    const ra = this.find(a);
    const rb = this.find(b);
    if (ra === rb) return false;
    if (this.rank[ra] < this.rank[rb]) this.parent[ra] = rb;
    else if (this.rank[ra] > this.rank[rb]) this.parent[rb] = ra;
    else {
      this.parent[rb] = ra;
      this.rank[ra] += 1;
    }
    return true;
  }
}

function kruskal(graph: Graph, edges: Edge[]): GraphResult {
  const sorted = [...edges].sort((a, b) => a.w - b.w || a.u - b.u || a.v - b.v);
  const dsu = new DSU(graph.labels.length);
  const chosen: Edge[] = [];
  const steps: GraphStep[] = [
    { text: `Sort the edges by weight: ${sorted.map((edge) => `${labelOf(graph, edge.u)}${labelOf(graph, edge.v)} (${edge.w})`).join(', ')}.` }
  ];
  let total = 0;
  for (const edge of sorted) {
    const accepted = dsu.union(edge.u, edge.v);
    if (accepted) {
      chosen.push(edge);
      total += edge.w;
      steps.push({
        text: `Accept $${labelOf(graph, edge.u)}${labelOf(graph, edge.v)}$ with weight ${edge.w}; running total ${total}.`
      });
    } else {
      steps.push({
        text: `Reject $${labelOf(graph, edge.u)}${labelOf(graph, edge.v)}$ with weight ${edge.w}; it would form a cycle.`
      });
    }
  }
  const connected = chosen.length === graph.labels.length - 1;
  const summary = [
    connected ? `Minimum spanning tree total: ${total}.` : `The graph is disconnected; a spanning tree is not possible.`,
    `Edges: ${chosen.map((edge) => `${labelOf(graph, edge.u)}${labelOf(graph, edge.v)} (${edge.w})`).join(', ') || 'none'}.`
  ];
  return {
    title: 'Kruskal minimum spanning tree',
    summary,
    steps,
    latex: documentLatex('Kruskal minimum spanning tree', graph, summary, steps, sorted)
  };
}

function prim(graph: Graph, edges: Edge[], start: number): GraphResult {
  const n = graph.labels.length;
  const visited = Array(n).fill(false);
  visited[start] = true;
  const chosen: Edge[] = [];
  const steps: GraphStep[] = [{ text: `Start at $${labelOf(graph, start)}$.` }];
  let total = 0;

  while (chosen.length < n - 1) {
    let best: Edge | null = null;
    for (const edge of edges) {
      const fromVisited = visited[edge.u] && !visited[edge.v];
      const reverse = !graph.directed && visited[edge.v] && !visited[edge.u];
      if ((fromVisited || reverse) && (!best || edge.w < best.w)) best = edge;
    }
    if (!best) break;
    const next = visited[best.u] ? best.v : best.u;
    visited[next] = true;
    chosen.push(best);
    total += best.w;
    steps.push({
      text: `Add $${labelOf(graph, best.u)}${labelOf(graph, best.v)}$ with weight ${best.w}; running total ${total}.`
    });
  }

  const connected = chosen.length === n - 1;
  const summary = [
    connected ? `Minimum spanning tree total: ${total}.` : `The graph is disconnected from ${labelOf(graph, start)}.`,
    `Edges: ${chosen.map((edge) => `${labelOf(graph, edge.u)}${labelOf(graph, edge.v)} (${edge.w})`).join(', ') || 'none'}.`
  ];
  return {
    title: 'Prim minimum spanning tree',
    summary,
    steps,
    latex: documentLatex('Prim minimum spanning tree', graph, summary, steps, edges)
  };
}

function documentLatex(
  title: string,
  graph: Graph,
  summary: string[],
  steps: GraphStep[],
  edges: Edge[]
): string {
  const edgeList = edges
    .map((edge) => `${labelOf(graph, edge.u)}${graph.directed ? '\\to' : '--'}${labelOf(graph, edge.v)}`
      + `\\;(\\text{weight }${edge.w})`)
    .join(', ');
  return `\\documentclass[11pt]{article}
\\usepackage[a4paper,margin=2.2cm]{geometry}
\\usepackage{amsmath,amssymb}
\\setlength{\\parindent}{0pt}
\\begin{document}
\\section*{${title}}
Nodes: ${graph.labels.join(', ')}. Edges: ${edgeList || 'none'}.

\\subsection*{Result}
${summary.map((line) => `${line.replace(/→/g, '\\to ')}\\\\`).join('\n')}

\\subsection*{Working}
\\begin{itemize}
${steps.map((step) => `\\item ${step.text}`).join('\n')}
\\end{itemize}
\\end{document}
`;
}
