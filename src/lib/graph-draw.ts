export interface DrawNode {
  id: string;
  label: string;
  x: number;
  y: number;
  color: string;
}

export interface DrawEdge {
  id: string;
  u: string;
  v: string;
  weight: string;
  color: string;
}

export interface DrawGraph {
  title: string;
  nodes: DrawNode[];
  edges: DrawEdge[];
  directed: boolean;
  weighted: boolean;
}

export const CANVAS_WIDTH = 760;
export const CANVAS_HEIGHT = 520;
export const NODE_RADIUS = 22;

export const PALETTE: { key: string; label: string; value: string; dark: string }[] = [
  { key: 'accent', label: 'Iris', value: '#b8a6f8', dark: '#3b2f63' },
  { key: 'green', label: 'Green', value: '#7ee787', dark: '#1f4d2b' },
  { key: 'amber', label: 'Amber', value: '#f0b72f', dark: '#5b4307' },
  { key: 'red', label: 'Red', value: '#ff7b72', dark: '#5e2420' },
  { key: 'blue', label: 'Blue', value: '#79c0ff', dark: '#183d63' }
];

const DEFAULT_NODE = '#0a0a0d';
const DEFAULT_STROKE = '#b8a6f8';
const DEFAULT_EDGE = '#7c8291';
const DEFAULT_TEXT = '#e4e6ec';

export function cloneGraph(graph: DrawGraph): DrawGraph {
  return {
    title: graph.title,
    directed: graph.directed,
    weighted: graph.weighted,
    nodes: graph.nodes.map((node) => ({ ...node })),
    edges: graph.edges.map((edge) => ({ ...edge }))
  };
}

export interface EdgeGeometry {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  labelX: number;
  labelY: number;
  arrow: string;
}

export function edgeGeometry(
  from: DrawNode,
  to: DrawNode,
  directed: boolean
): EdgeGeometry {
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  const length = Math.hypot(dx, dy) || 1;
  const ux = dx / length;
  const uy = dy / length;
  const startX = from.x + ux * (NODE_RADIUS + 2);
  const startY = from.y + uy * (NODE_RADIUS + 2);
  const endX = to.x - ux * (NODE_RADIUS + 2);
  const endY = to.y - uy * (NODE_RADIUS + 2);
  const midX = (startX + endX) / 2;
  const midY = (startY + endY) / 2;
  const perpX = -uy;
  const perpY = ux;
  const labelX = midX + perpX * 12;
  const labelY = midY + perpY * 12;
  const arrow = directed
    ? [
        `${endX},${endY}`,
        `${endX - ux * 13 + perpX * 5},${endY - uy * 13 + perpY * 5}`,
        `${endX - ux * 13 - perpX * 5},${endY - uy * 13 - perpY * 5}`
      ].join(' ')
    : '';
  return { x1: startX, y1: startY, x2: endX, y2: endY, labelX, labelY, arrow };
}

function escapeXml(value: string): string {
  return String(value)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function escapeTex(value: string): string {
  return String(value).replace(/[\\&%$#_{}~^]/g, (character) => {
    const map: Record<string, string> = {
      '\\': '\\textbackslash{}',
      '&': '\\&',
      '%': '\\%',
      $: '\\$',
      '#': '\\#',
      _: '\\_',
      '{': '\\{',
      '}': '\\}',
      '~': '\\textasciitilde{}',
      '^': '\\textasciicircum{}'
    };
    return map[character] ?? character;
  });
}

export function buildSvg(graph: DrawGraph): string {
  const byId = new Map(graph.nodes.map((node) => [node.id, node]));
  const edgeMarkup = graph.edges
    .map((edge) => {
      const from = byId.get(edge.u);
      const to = byId.get(edge.v);
      if (!from || !to) return '';
      const geometry = edgeGeometry(from, to, graph.directed);
      const color = edge.color || DEFAULT_EDGE;
      const weight =
        graph.weighted && edge.weight.trim()
          ? `<text x="${geometry.labelX.toFixed(2)}" y="${geometry.labelY.toFixed(2)}" fill="${DEFAULT_TEXT}" stroke="${DEFAULT_NODE}" stroke-width="4" paint-order="stroke" font-size="13" font-weight="600" text-anchor="middle" dominant-baseline="middle">${escapeXml(edge.weight)}</text>`
          : '';
      const arrow = geometry.arrow
        ? `<polygon points="${geometry.arrow}" fill="${color}"/>`
        : '';
      return `<g><line x1="${geometry.x1.toFixed(2)}" y1="${geometry.y1.toFixed(2)}" x2="${geometry.x2.toFixed(2)}" y2="${geometry.y2.toFixed(2)}" stroke="${color}" stroke-width="2.4" stroke-linecap="round"/>${arrow}${weight}</g>`;
    })
    .join('\n');

  const nodeMarkup = graph.nodes
    .map((node) => {
      const fill = node.color || DEFAULT_NODE;
      const stroke = node.color ? '#f4ead1' : DEFAULT_STROKE;
      const text = node.color ? '#0a0a0d' : DEFAULT_TEXT;
      return `<g><circle cx="${node.x.toFixed(2)}" cy="${node.y.toFixed(2)}" r="${NODE_RADIUS}" fill="${fill}" stroke="${stroke}" stroke-width="2"/><text x="${node.x.toFixed(2)}" y="${node.y.toFixed(2)}" fill="${text}" font-size="15" font-weight="700" text-anchor="middle" dominant-baseline="middle">${escapeXml(node.label)}</text></g>`;
    })
    .join('\n');

  return `<svg xmlns="http://www.w3.org/2000/svg" width="${CANVAS_WIDTH}" height="${CANVAS_HEIGHT}" viewBox="0 0 ${CANVAS_WIDTH} ${CANVAS_HEIGHT}">
<rect width="100%" height="100%" fill="${DEFAULT_NODE}"/>
<text x="${CANVAS_WIDTH / 2}" y="30" fill="${DEFAULT_TEXT}" font-size="18" font-weight="700" text-anchor="middle">${escapeXml(graph.title)}</text>
<g font-family="Hubot Sans, Arial, sans-serif">
${edgeMarkup}
${nodeMarkup}
</g>
</svg>`;
}

export function buildTikz(graph: DrawGraph): string {
  const index = new Map(graph.nodes.map((node, position) => [node.id, position]));
  const lines = [
    '% Generated by lab.faris0x.dev graph drawer',
    '\\begin{tikzpicture}[scale=0.018, every node/.style={circle,draw,minimum size=22pt,inner sep=0pt}]'
  ];
  for (const node of graph.nodes) {
    lines.push(
      `  \\node (n${index.get(node.id)}) at (${node.x.toFixed(1)},${(-node.y).toFixed(1)}) {${escapeTex(node.label)}};`
    );
  }
  for (const edge of graph.edges) {
    const u = index.get(edge.u);
    const v = index.get(edge.v);
    if (u === undefined || v === undefined) continue;
    const arrow = graph.directed ? '[->]' : '';
    const label =
      graph.weighted && edge.weight.trim()
        ? ` node[midway,above,draw=none] {${escapeTex(edge.weight)}}`
        : '';
    lines.push(`  \\draw${arrow} (n${u}) --${label} (n${v});`);
  }
  lines.push('\\end{tikzpicture}');
  return lines.join('\n');
}

export function downloadText(filename: string, text: string, type: string): void {
  const blob = new Blob([text], { type });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  link.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function graphToJson(graph: DrawGraph): string {
  return JSON.stringify({ version: 1, ...graph }, null, 2);
}

export function parseGraphJson(source: string): DrawGraph {
  const value = JSON.parse(source) as Partial<DrawGraph>;
  if (
    !value ||
    !Array.isArray(value.nodes) ||
    !Array.isArray(value.edges) ||
    typeof value.title !== 'string'
  ) {
    throw new Error('This file is not a graph drawing.');
  }
  const nodes = value.nodes.map((node, index) => {
    if (
      !node ||
      typeof node.id !== 'string' ||
      typeof node.label !== 'string' ||
      typeof node.x !== 'number' ||
      typeof node.y !== 'number'
    ) {
      throw new Error(`Node ${index + 1} is invalid.`);
    }
    return {
      id: node.id,
      label: node.label,
      x: node.x,
      y: node.y,
      color: typeof node.color === 'string' ? node.color : ''
    };
  });
  const ids = new Set(nodes.map((node) => node.id));
  const edges = value.edges.map((edge, index) => {
    if (
      !edge ||
      typeof edge.id !== 'string' ||
      typeof edge.u !== 'string' ||
      typeof edge.v !== 'string' ||
      typeof edge.weight !== 'string'
    ) {
      throw new Error(`Edge ${index + 1} is invalid.`);
    }
    if (!ids.has(edge.u) || !ids.has(edge.v)) {
      throw new Error(`Edge ${index + 1} points at a missing node.`);
    }
    return {
      id: edge.id,
      u: edge.u,
      v: edge.v,
      weight: edge.weight,
      color: typeof edge.color === 'string' ? edge.color : ''
    };
  });
  return {
    title: value.title,
    nodes,
    edges,
    directed: Boolean(value.directed),
    weighted: value.weighted !== false
  };
}
