export type Node =
  | { type: 'var'; name: string }
  | { type: 'not'; child: Node }
  | { type: 'and' | 'or' | 'xor' | 'imp' | 'iff'; left: Node; right: Node };

interface Token {
  kind: 'var' | 'not' | 'and' | 'or' | 'xor' | 'imp' | 'iff' | 'open' | 'close';
  value: string;
}

export interface TableColumn {
  label: string;
  key: string;
}

export interface TableRow {
  values: boolean[];
}

export interface LogicResult {
  formula: string;
  variables: string[];
  columns: TableColumn[];
  rows: TableRow[];
  verdict: 'tautology' | 'contradiction' | 'contingent';
  latex: string;
}

function tokenize(source: string): Token[] {
  const text = source
    .replace(/[¬]/g, '~')
    .replace(/[∧]/g, '&')
    .replace(/[∨]/g, '|')
    .replace(/[→]/g, '->')
    .replace(/[↔]/g, '<->')
    .replace(/\s+/g, '');

  const tokens: Token[] = [];
  let i = 0;
  while (i < text.length) {
    const rest = text.slice(i);
    if (rest.startsWith('<->')) {
      tokens.push({ kind: 'iff', value: '<->' });
      i += 3;
    } else if (rest.startsWith('->')) {
      tokens.push({ kind: 'imp', value: '->' });
      i += 2;
    } else {
      const ch = text[i];
      if (ch === '~' || ch === '!') tokens.push({ kind: 'not', value: ch });
      else if (ch === '&') tokens.push({ kind: 'and', value: ch });
      else if (ch === '^') tokens.push({ kind: 'xor', value: ch });
      else if (ch === '|' || ch === 'v') tokens.push({ kind: 'or', value: ch });
      else if (ch === '(') tokens.push({ kind: 'open', value: ch });
      else if (ch === ')') tokens.push({ kind: 'close', value: ch });
      else if (/[a-z]/i.test(ch)) tokens.push({ kind: 'var', value: ch.toLowerCase() });
      else if (ch === '⊤' || ch === '1') tokens.push({ kind: 'var', value: 't' });
      else if (ch === '⊥' || ch === '0') tokens.push({ kind: 'var', value: 'f' });
      else throw new Error(`Unknown symbol “${ch}”. Use letters, ~, &, |, ->, <-> and parentheses.`);
      i += 1;
    }
  }
  if (!tokens.length) throw new Error('Enter a formula.');
  return tokens;
}

export function parse(source: string): Node {
  const tokens = tokenize(source);
  let index = 0;

  function peek(): Token | undefined {
    return tokens[index];
  }

  function take(kind?: Token['kind']): Token {
    const token = tokens[index];
    if (!token) throw new Error('The formula ends early.');
    if (kind && token.kind !== kind) {
      throw new Error(`Expected ${kind} but found “${token.value}”.`);
    }
    index += 1;
    return token;
  }

  function parseIff(): Node {
    let left = parseImp();
    while (peek()?.kind === 'iff') {
      take('iff');
      left = { type: 'iff', left, right: parseImp() };
    }
    return left;
  }

  function parseImp(): Node {
    const left = parseOr();
    if (peek()?.kind === 'imp') {
      take('imp');
      return { type: 'imp', left, right: parseImp() };
    }
    return left;
  }

  function parseOr(): Node {
    let left = parseXor();
    while (peek()?.kind === 'or') {
      take('or');
      left = { type: 'or', left, right: parseXor() };
    }
    return left;
  }

  function parseXor(): Node {
    let left = parseAnd();
    while (peek()?.kind === 'xor') {
      take('xor');
      left = { type: 'xor', left, right: parseAnd() };
    }
    return left;
  }

  function parseAnd(): Node {
    let left = parseNot();
    while (peek()?.kind === 'and') {
      take('and');
      left = { type: 'and', left, right: parseNot() };
    }
    return left;
  }

  function parseNot(): Node {
    if (peek()?.kind === 'not') {
      take('not');
      return { type: 'not', child: parseNot() };
    }
    return parseAtom();
  }

  function parseAtom(): Node {
    const token = peek();
    if (!token) throw new Error('The formula ends early.');
    if (token.kind === 'open') {
      take('open');
      const inner = parseIff();
      take('close');
      return inner;
    }
    if (token.kind === 'var') {
      take('var');
      return { type: 'var', name: token.value };
    }
    throw new Error(`Unexpected “${token.value}”.`);
  }

  const tree = parseIff();
  if (index < tokens.length) {
    throw new Error(`Unexpected “${tokens[index].value}” after the formula.`);
  }
  return tree;
}

export function label(node: Node): string {
  switch (node.type) {
    case 'var':
      return node.name;
    case 'not':
      return `¬${needsParens(node.child) ? `(${label(node.child)})` : label(node.child)}`;
    case 'and':
      return `${wrap(node.left)} ∧ ${wrap(node.right)}`;
    case 'or':
      return `${wrap(node.left)} ∨ ${wrap(node.right)}`;
    case 'xor':
      return `${wrap(node.left)} ⊕ ${wrap(node.right)}`;
    case 'imp':
      return `${wrap(node.left)} → ${wrap(node.right)}`;
    case 'iff':
      return `${wrap(node.left)} ↔ ${wrap(node.right)}`;
  }
}

function needsParens(node: Node): boolean {
  return node.type === 'and' || node.type === 'or' || node.type === 'xor' || node.type === 'imp' || node.type === 'iff';
}

function wrap(node: Node): string {
  return needsParens(node) ? `(${label(node)})` : label(node);
}

export function evaluate(node: Node, values: Record<string, boolean>): boolean {
  switch (node.type) {
    case 'var':
      return node.name === 't' ? true : node.name === 'f' ? false : Boolean(values[node.name]);
    case 'not':
      return !evaluate(node.child, values);
    case 'and':
      return evaluate(node.left, values) && evaluate(node.right, values);
    case 'or':
      return evaluate(node.left, values) || evaluate(node.right, values);
    case 'xor':
      return evaluate(node.left, values) !== evaluate(node.right, values);
    case 'imp':
      return !evaluate(node.left, values) || evaluate(node.right, values);
    case 'iff':
      return evaluate(node.left, values) === evaluate(node.right, values);
  }
}

export function build(source: string): LogicResult {
  const tree = parse(source);
  const variables = new Set<string>();
  const subexpressions: Node[] = [];

  function walk(node: Node, depth: number): number {
    let height = depth;
    if (node.type === 'var') {
      if (node.name !== 't' && node.name !== 'f') variables.add(node.name);
    } else {
      if (node.type === 'not') height = Math.max(height, walk(node.child, depth + 1));
      else height = Math.max(height, walk(node.left, depth + 1), walk(node.right, depth + 1));
      subexpressions.push(node);
    }
    return height;
  }
  walk(tree, 0);

  const variableNames = [...variables].sort();
  const unique: { node: Node; label: string; key: string }[] = [];
  for (const node of subexpressions) {
    const text = label(node);
    if (!unique.some((entry) => entry.key === text)) {
      unique.push({ node, label: text, key: text });
    }
  }
  unique.sort((a, b) => {
    const da = depth(a.node);
    const db = depth(b.node);
    return da === db ? a.key.length - b.key.length : da - db;
  });

  if (tree.type !== 'var' || tree.name === 't' || tree.name === 'f') {
    const rootKey = label(tree);
    const rootEntry = { node: tree, label: rootKey, key: rootKey };
    const others = unique.filter((entry) => entry.key !== rootKey || entry.node !== tree);
    unique.length = 0;
    unique.push(...others, rootEntry);
  }

  const columns: TableColumn[] = [
    ...variableNames.map((name) => ({ label: name, key: name })),
    ...unique.map((entry) => ({ label: entry.label, key: entry.key }))
  ];

  const rows: TableRow[] = [];
  const total = 1 << variableNames.length;
  for (let mask = 0; mask < total; mask += 1) {
    const values: Record<string, boolean> = {};
    variableNames.forEach((name, bit) => {
      values[name] = Boolean(mask & (1 << (variableNames.length - 1 - bit)));
    });
    const rowValues = [
      ...variableNames.map((name) => values[name]),
      ...unique.map((entry) => evaluate(entry.node, values))
    ];
    rows.push({ values: rowValues });
  }

  const finalIndex = columns.length - 1;
  const finals = rows.map((row) => row.values[finalIndex]);
  const verdict = finals.every(Boolean)
    ? 'tautology'
    : finals.every((value) => !value)
      ? 'contradiction'
      : 'contingent';

  return {
    formula: source.trim(),
    variables: variableNames,
    columns,
    rows,
    verdict,
    latex: buildLatex(source.trim(), columns, rows, verdict)
  };
}

function depth(node: Node): number {
  if (node.type === 'var') return 0;
  if (node.type === 'not') return 1 + depth(node.child);
  return 1 + Math.max(depth(node.left), depth(node.right));
}

function buildLatex(
  formula: string,
  columns: TableColumn[],
  rows: TableRow[],
  verdict: LogicResult['verdict']
): string {
  const texLabel = (value: string) =>
    value
      .replace(/¬/g, '\\neg ')
      .replace(/∧/g, '\\land ')
      .replace(/∨/g, '\\lor ')
      .replace(/→/g, '\\to ')
      .replace(/↔/g, '\\leftrightarrow ');
  const header = columns
    .map((column) => texLabel(column.label))
    .join(' & ');
  const body = rows
    .map((row) => row.values.map((value) => (value ? '\\top' : '\\bot')).join(' & '))
    .join(' \\\\\n');
  const verdictText =
    verdict === 'tautology'
      ? 'The formula is a tautology: every row is true.'
      : verdict === 'contradiction'
        ? 'The formula is a contradiction: every row is false.'
        : 'The formula is contingent: it is true in some rows and false in others.';

  return `\\documentclass[11pt]{article}
\\usepackage[a4paper,margin=2.2cm]{geometry}
\\usepackage{amsmath,amssymb}
\\usepackage{array}
\\setlength{\\parindent}{0pt}
\\begin{document}
\\section*{Truth table}
\\[
${formula
  .replace(/<->/g, '\\leftrightarrow ')
  .replace(/->/g, '\\to ')
  .replace(/[&∧]/g, '\\land ')
   .replace(/[|∨]/g, '\\lor ')
  .replace(/\^/g, '\\oplus ')
  .replace(/[~¬]/g, '\\neg ')}
\\]
\\[
\\begin{array}{${'c|'.repeat(columns.length - 1)}c}
${header} \\\\ \\hline
${body}
\\end{array}
\\]
${verdictText}
\\end{document}
`;
}
