import { describe, expect, it } from 'vitest';
import {
  buildSvg,
  buildTikz,
  edgeGeometry,
  parseGraphJson,
  weightLabelWidth,
  type DrawEdge,
  type DrawGraph,
  type DrawNode
} from './graph-draw';

const nodes: DrawNode[] = [
  { id: 'a', label: 'A', x: 100, y: 100, color: '' },
  { id: 'b', label: 'B', x: 300, y: 100, color: '' }
];

const edge: DrawEdge = {
  id: 'e',
  u: 'a',
  v: 'b',
  weight: '4',
  color: '',
  directed: true,
  weightT: 0.5,
  weightOffset: 0
};

const graph: DrawGraph = {
  title: 'Test graph',
  nodes,
  edges: [edge],
  weighted: true,
  edgeFontSize: 14,
  nodeFontSize: 16
};

describe('graph export', () => {
  it('always exports black on white', () => {
    const svg = buildSvg(graph);
    expect(svg).toContain('fill="#ffffff"');
    expect(svg).toContain('stroke="#000000"');
    expect(svg).toContain('fill="#000000"');
    expect(svg).not.toContain('#0a0a0d');
  });

  it('embeds a supplied font stylesheet', () => {
    const svg = buildSvg(graph, undefined, "font-family:'Lab Paper';");
    expect(svg).toContain("font-family:'Lab Paper';");
  });

  it('leaves a gap in the line for the weight', () => {
    const geometry = edgeGeometry(
      nodes[0],
      nodes[1],
      edge,
      weightLabelWidth('4', graph.edgeFontSize)
    );
    expect(geometry.line1EndX).toBeLessThan(geometry.line2StartX);
  });

  it('writes inline TikZ weights and directed arrows', () => {
    const tikz = buildTikz(graph);
    expect(tikz).toContain('fill=white');
    expect(tikz).toContain('[->]');
    expect(tikz).toContain('\\usepackage{tikz}');
  });

  it('migrates the old global-directed JSON format', () => {
    const parsed = parseGraphJson(
      JSON.stringify({
        title: 'Old',
        directed: true,
        weighted: true,
        nodes,
        edges: [{ id: 'e', u: 'a', v: 'b', weight: '4', color: '' }]
      })
    );
    expect(parsed.edges[0].directed).toBe(true);
    expect(parsed.edges[0].weightT).toBe(0.5);
    expect(parsed.edgeFontSize).toBe(14);
  });
});
