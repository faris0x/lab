// Copyright (c) 2026 Faris Alfarhan
// SPDX-License-Identifier: GPL-3.0-only

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
  directed: boolean;
  weightT: number;
  weightOffset: number;
}

export interface DrawGraph {
  title: string;
  nodes: DrawNode[];
  edges: DrawEdge[];
  weighted: boolean;
  edgeFontSize: number;
  nodeFontSize: number;
}

export const CANVAS_WIDTH = 760;
export const CANVAS_HEIGHT = 520;
export const NODE_RADIUS = 22;
export const DEFAULT_EDGE_FONT = 14;
export const DEFAULT_NODE_FONT = 16;

export const PALETTE: { key: string; label: string; value: string; dark: string }[] = [
  { key: 'accent', label: 'Iris', value: '#b8a6f8', dark: '#3b2f63' },
  { key: 'green', label: 'Green', value: '#7ee787', dark: '#1f4d2b' },
  { key: 'amber', label: 'Amber', value: '#f0b72f', dark: '#5b4307' },
  { key: 'red', label: 'Red', value: '#ff7b72', dark: '#5e2420' },
  { key: 'blue', label: 'Blue', value: '#79c0ff', dark: '#183d63' }
];

export interface RenderTheme {
  background: string;
  ink: string;
  nodeFill: string;
  nodeStroke: string;
  edge: string;
}

export const EXPORT_THEME: RenderTheme = {
  background: '#ffffff',
  ink: '#000000',
  nodeFill: '#ffffff',
  nodeStroke: '#000000',
  edge: '#000000'
};

export function cloneGraph(graph: DrawGraph): DrawGraph {
  return {
    title: graph.title,
    weighted: graph.weighted,
    edgeFontSize: graph.edgeFontSize,
    nodeFontSize: graph.nodeFontSize,
    nodes: graph.nodes.map((node) => ({ ...node })),
    edges: graph.edges.map((edge) => ({ ...edge }))
  };
}

export function weightLabelWidth(text: string, fontSize: number): number {
  const trimmed = text.trim();
  if (!trimmed) return 0;
  return Math.max(10, trimmed.length * fontSize * 0.62 + 8);
}

export interface EdgeGeometry {
  startX: number;
  startY: number;
  endX: number;
  endY: number;
  line1EndX: number;
  line1EndY: number;
  line2StartX: number;
  line2StartY: number;
  labelX: number;
  labelY: number;
  arrow: string;
}

export function edgeGeometry(
  from: DrawNode,
  to: DrawNode,
  edge: DrawEdge,
  labelWidth: number
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
  const lineLength = Math.hypot(endX - startX, endY - startY) || 1;
  const perpX = -uy;
  const perpY = ux;
  const t = Math.max(0.05, Math.min(0.95, edge.weightT ?? 0.5));
  const along = lineLength * t;
  const labelX = startX + ux * along + perpX * (edge.weightOffset ?? 0);
  const labelY = startY + uy * along + perpY * (edge.weightOffset ?? 0);
  const halfGap = labelWidth
    ? Math.min(labelWidth / 2 + 4, lineLength / 2 - 1)
    : 0;
  const gapStart = Math.max(0, Math.min(lineLength, along - halfGap));
  const gapEnd = Math.max(0, Math.min(lineLength, along + halfGap));
  const arrow = edge.directed
    ? [
        `${endX},${endY}`,
        `${endX - ux * 13 + perpX * 5},${endY - uy * 13 + perpY * 5}`,
        `${endX - ux * 13 - perpX * 5},${endY - uy * 13 - perpY * 5}`
      ].join(' ')
    : '';
  return {
    startX,
    startY,
    endX,
    endY,
    line1EndX: startX + ux * gapStart,
    line1EndY: startY + uy * gapStart,
    line2StartX: startX + ux * gapEnd,
    line2StartY: startY + uy * gapEnd,
    labelX,
    labelY,
    arrow
  };
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

export function buildSvg(
  graph: DrawGraph,
  theme: RenderTheme = EXPORT_THEME,
  fontCss = ''
): string {
  const byId = new Map(graph.nodes.map((node) => [node.id, node]));
  const edgeMarkup = graph.edges
    .map((edge) => {
      const from = byId.get(edge.u);
      const to = byId.get(edge.v);
      if (!from || !to) return '';
      const weightText = graph.weighted && edge.weight.trim() ? edge.weight : '';
      const labelWidth = weightText
        ? weightLabelWidth(weightText, graph.edgeFontSize)
        : 0;
      const geometry = edgeGeometry(from, to, edge, labelWidth);
      const weight = weightText
        ? `<text x="${geometry.labelX.toFixed(2)}" y="${geometry.labelY.toFixed(2)}" fill="${theme.ink}" font-size="${graph.edgeFontSize}" font-weight="400" text-anchor="middle" dominant-baseline="middle">${escapeXml(weightText)}</text>`
        : '';
      const arrow = geometry.arrow
        ? `<polygon points="${geometry.arrow}" fill="${theme.ink}"/>`
        : '';
      return `<g><line x1="${geometry.startX.toFixed(2)}" y1="${geometry.startY.toFixed(2)}" x2="${geometry.line1EndX.toFixed(2)}" y2="${geometry.line1EndY.toFixed(2)}" stroke="${theme.edge}" stroke-width="1.6" stroke-linecap="round"/><line x1="${geometry.line2StartX.toFixed(2)}" y1="${geometry.line2StartY.toFixed(2)}" x2="${geometry.endX.toFixed(2)}" y2="${geometry.endY.toFixed(2)}" stroke="${theme.edge}" stroke-width="1.6" stroke-linecap="round"/>${arrow}${weight}</g>`;
    })
    .join('\n');

  const nodeMarkup = graph.nodes
    .map((node) => {
      return `<g><circle cx="${node.x.toFixed(2)}" cy="${node.y.toFixed(2)}" r="${NODE_RADIUS}" fill="${theme.nodeFill}" stroke="${theme.nodeStroke}" stroke-width="1.6"/><text x="${node.x.toFixed(2)}" y="${node.y.toFixed(2)}" fill="${theme.ink}" font-size="${graph.nodeFontSize}" font-weight="400" font-style="italic" text-anchor="middle" dominant-baseline="middle">${escapeXml(node.label)}</text></g>`;
    })
    .join('\n');

  const fontStyle = fontCss
    ? `<defs><style type="text/css"><![CDATA[${fontCss}]]></style></defs>`
    : '';

  return `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="${CANVAS_WIDTH}" height="${CANVAS_HEIGHT}" viewBox="0 0 ${CANVAS_WIDTH} ${CANVAS_HEIGHT}">
${fontStyle}
<rect width="100%" height="100%" fill="${theme.background}"/>
<g font-family="Lab Paper, 'Latin Modern Roman', 'CMU Serif', 'Times New Roman', serif">
<text x="${CANVAS_WIDTH / 2}" y="30" fill="${theme.ink}" font-size="18" font-weight="700" text-anchor="middle">${escapeXml(graph.title)}</text>
${edgeMarkup}
${nodeMarkup}
</g>
</svg>`;
}

function tikzValue(value: string): string {
  const trimmed = value.trim();
  if (/^[A-Za-z0-9+\-./*=() ]+$/.test(trimmed)) return `$${trimmed}$`;
  return escapeTex(trimmed);
}

export function buildTikz(graph: DrawGraph): string {
  const index = new Map(graph.nodes.map((node, position) => [node.id, position]));
  const nodeFont = Math.max(8, Math.round(graph.nodeFontSize * 0.75));
  const edgeFont = Math.max(7, Math.round(graph.edgeFontSize * 0.75));
  const lines = [
    '% Generated by lab.faris0x.dev graph drawer',
    '% Requires \\usepackage{tikz}',
    `\\begin{tikzpicture}[>=stealth,line width=0.5pt,every node/.style={circle,draw,inner sep=1.5pt,minimum size=22pt,font=\\fontsize{${nodeFont}}{${nodeFont + 2}}\\selectfont}]`
  ];
  for (const node of graph.nodes) {
    lines.push(
      `  \\node (n${index.get(node.id)}) at (${node.x.toFixed(1)},${(-node.y).toFixed(1)}) {${tikzValue(node.label)}};`
    );
  }
  for (const edge of graph.edges) {
    const u = index.get(edge.u);
    const v = index.get(edge.v);
    if (u === undefined || v === undefined) continue;
    const arrow = edge.directed ? '[->]' : '';
    const direction =
      (edge.weightOffset ?? 0) < 0
        ? ',below'
        : (edge.weightOffset ?? 0) > 0
          ? ',above'
          : '';
    const label =
      graph.weighted && edge.weight.trim()
        ? ` node[pos=${Math.max(0.05, Math.min(0.95, edge.weightT ?? 0.5)).toFixed(2)},fill=white,inner sep=1.5pt${direction},font=\\fontsize{${edgeFont}}{${edgeFont + 2}}\\selectfont] {${tikzValue(edge.weight)}}`
        : '';
    lines.push(`  \\draw${arrow} (n${u}) --${label} (n${v});`);
  }
  lines.push('\\end{tikzpicture}');
  return lines.join('\n');
}

export function downloadBlob(filename: string, blob: Blob): void {
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  link.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function downloadText(filename: string, text: string, type: string): void {
  downloadBlob(filename, new Blob([text], { type }));
}

export function svgToPngBlob(svg: string, scale = 2): Promise<Blob> {
  return new Promise((resolve, reject) => {
    const source = new Blob([svg], { type: 'image/svg+xml' });
    const url = URL.createObjectURL(source);
    const image = new Image();
    image.onload = () => {
      const canvas = document.createElement('canvas');
      canvas.width = CANVAS_WIDTH * scale;
      canvas.height = CANVAS_HEIGHT * scale;
      const context = canvas.getContext('2d');
      if (!context) {
        URL.revokeObjectURL(url);
        reject(new Error('This browser cannot create a PNG canvas.'));
        return;
      }
      context.drawImage(image, 0, 0, canvas.width, canvas.height);
      canvas.toBlob((blob) => {
        URL.revokeObjectURL(url);
        if (blob) resolve(blob);
        else reject(new Error('This browser could not encode the PNG.'));
      }, 'image/png');
    };
    image.onerror = () => {
      URL.revokeObjectURL(url);
      reject(new Error('The graph could not be rasterised.'));
    };
    image.src = url;
  });
}

export function graphToJson(graph: DrawGraph): string {
  return JSON.stringify({ version: 1, ...graph }, null, 2);
}

export function parseGraphJson(source: string): DrawGraph {
  const value = JSON.parse(source) as Partial<DrawGraph> & {
    directed?: boolean;
    edges?: Partial<DrawEdge>[];
  };
  if (
    !value ||
    !Array.isArray(value.nodes) ||
    !Array.isArray(value.edges) ||
    typeof value.title !== 'string'
  ) {
    throw new Error('This file is not a graph drawing.');
  }
  const legacyDirected = Boolean(value.directed);
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
      color: typeof edge.color === 'string' ? edge.color : '',
      directed: typeof edge.directed === 'boolean' ? edge.directed : legacyDirected,
      weightT: typeof edge.weightT === 'number' ? edge.weightT : 0.5,
      weightOffset: typeof edge.weightOffset === 'number' ? edge.weightOffset : 0
    };
  });
  return {
    title: value.title,
    nodes,
    edges,
    weighted: value.weighted !== false,
    edgeFontSize:
      typeof value.edgeFontSize === 'number' ? value.edgeFontSize : DEFAULT_EDGE_FONT,
    nodeFontSize:
      typeof value.nodeFontSize === 'number' ? value.nodeFontSize : DEFAULT_NODE_FONT
  };
}
