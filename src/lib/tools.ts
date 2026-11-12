export interface ToolMeta {
  key: string;
  href: string;
  nav: string;
  navAr?: string;
  title: string;
  tag: string;
  description: string;
}

/** One source of truth for the nav, home cards and page metadata. */
export const tools: ToolMeta[] = [
  {
    key: 'simplex',
    href: '/optimisation',
    nav: 'Simplex',
    navAr: 'السمبلكس',
    title: 'Simplex & Big-M',
    tag: 'Live · Optimisation',
    description:
      'Set up a linear program and move through its tableaus. The arithmetic stays exact; Big-M uses a symbolic penalty. The worked solution exports as a .tex file.'
  },
  {
    key: 'matrix',
    href: '/matrix',
    nav: 'Matrix',
    navAr: 'المصفوفات',
    title: 'Matrix toolkit',
    tag: 'Live · Linear algebra',
    description:
      'Row-reduce a matrix and inspect each operation. Determinant, inverse, nullspace, and linear systems use the same exact arithmetic.'
  },
  {
    key: 'complex',
    href: '/complex',
    nav: 'Complex',
    navAr: 'الأعداد المركبة',
    title: 'Complex numbers & Argand',
    tag: 'Live · Complex numbers',
    description:
      'Add, subtract, multiply, or divide complex numbers. Convert to polar form, use de Moivre for powers, find all roots, and see every result on an Argand diagram.'
  },
  {
    key: 'series',
    href: '/series',
    nav: 'Series',
    title: 'Taylor & Maclaurin series',
    tag: 'Live · Series',
    description:
      'Expand a function about a point. Every derivative, value and coefficient is shown, and the finished series exports as a .tex file.'
  },
  {
    key: 'logic',
    href: '/logic',
    nav: 'Logic',
    title: 'Truth tables',
    tag: 'Live · Logic',
    description:
      'Parse a propositional formula, evaluate every assignment and classify the result as a tautology, contradiction or contingent. The table exports as a .tex file.'
  },
  {
    key: 'graphs',
    href: '/graphs',
    nav: 'Graphs',
    title: 'Graphs & networks',
    tag: 'Live · Networks',
    description:
      'Edit a weighted graph and run Dijkstra, Floyd–Warshall, Kruskal or Prim. Distance updates, accepted edges and rejected cycles are recorded step by step.'
  },
  {
    key: 'float',
    href: '/floating-point',
    nav: 'Float',
    title: 'Floating point',
    tag: 'Live · Number systems',
    description:
      'Convert between binary, octal, denary and hexadecimal, normalise binary values, and encode or decode custom and IEEE 754 formats.'
  },
  {
    key: 'boolean',
    href: '/boolean',
    nav: 'Boolean',
    title: 'Boolean algebra',
    tag: 'Live · Boolean',
    description:
      'Build truth tables, canonical SOP and POS forms, prime implicants and a minimal sum of products for a Boolean statement.'
  }
];

export const toolKeys = tools.map((tool) => tool.key);

export function toolByKey(key: string): ToolMeta | undefined {
  return tools.find((tool) => tool.key === key);
}
