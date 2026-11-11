import { evaluate, label, parse, type Node } from './logic';

export interface BooleanRow {
  values: boolean[];
}

export interface PrimeImplicant {
  bits: string;
  expression: string;
  covers: number[];
}

export interface BooleanResult {
  formula: string;
  variables: string[];
  rows: BooleanRow[];
  minterms: number[];
  maxterms: number[];
  canonicalSop: string;
  canonicalPos: string;
  primes: PrimeImplicant[];
  essential: PrimeImplicant[];
  selected: PrimeImplicant[];
  minimal: string;
  latex: string;
}

function collect(node: Node, variables: Set<string>): void {
  if (node.type === 'var') {
    if (node.name !== 't' && node.name !== 'f') variables.add(node.name);
    return;
  }
  if (node.type === 'not') collect(node.child, variables);
  else {
    collect(node.left, variables);
    collect(node.right, variables);
  }
}

function binary(value: number, width: number): string {
  return value.toString(2).padStart(width, '0');
}

function termExpression(bits: string, variables: string[]): string {
  const parts: string[] = [];
  bits.split('').forEach((bit, index) => {
    if (bit === '-') return;
    parts.push(bit === '1' ? variables[index] : `¬${variables[index]}`);
  });
  return parts.length ? parts.join(' ∧ ') : '1';
}

function combine(a: string, b: string): string | null {
  let diff = -1;
  for (let i = 0; i < a.length; i += 1) {
    if (a[i] === b[i]) continue;
    if (diff !== -1) return null;
    diff = i;
  }
  if (diff === -1) return null;
  return `${a.slice(0, diff)}-${a.slice(diff + 1)}`;
}

function coversTerm(bits: string, minterm: number, width: number): boolean {
  const value = binary(minterm, width);
  for (let i = 0; i < width; i += 1) {
    if (bits[i] !== '-' && bits[i] !== value[i]) return false;
  }
  return true;
}

function minimise(minterms: number[], width: number, variables: string[]) {
  if (minterms.length === 0) {
    return { primes: [], essential: [], selected: [] };
  }
  if (minterms.length === 1 << width) {
    const prime: PrimeImplicant = {
      bits: '-'.repeat(width),
      expression: '1',
      covers: minterms.slice()
    };
    return { primes: [prime], essential: [prime], selected: [prime] };
  }

  let current = [...new Set(minterms.map((value) => binary(value, width)))];
  const primeBits = new Set<string>();

  while (current.length) {
    const used = new Set<string>();
    const next = new Set<string>();
    for (let i = 0; i < current.length; i += 1) {
      for (let j = i + 1; j < current.length; j += 1) {
        const merged = combine(current[i], current[j]);
        if (merged) {
          used.add(current[i]);
          used.add(current[j]);
          next.add(merged);
        }
      }
    }
    for (const term of current) if (!used.has(term)) primeBits.add(term);
    current = [...next];
  }

  const primes: PrimeImplicant[] = [...primeBits]
    .sort((a, b) => a.localeCompare(b))
    .map((bits) => ({
      bits,
      expression: termExpression(bits, variables),
      covers: minterms.filter((value) => coversTerm(bits, value, width))
    }));

  const essential: PrimeImplicant[] = [];
  const selected: PrimeImplicant[] = [];
  const uncovered = new Set(minterms);

  for (const minterm of minterms) {
    const covering = primes.filter((prime) => prime.covers.includes(minterm));
    if (covering.length === 1 && !selected.includes(covering[0])) {
      selected.push(covering[0]);
      essential.push(covering[0]);
      covering[0].covers.forEach((value) => uncovered.delete(value));
    }
  }

  while (uncovered.size) {
    let best: PrimeImplicant | null = null;
    let bestCount = 0;
    for (const prime of primes) {
      const count = prime.covers.filter((value) => uncovered.has(value)).length;
      if (count > bestCount) {
        best = prime;
        bestCount = count;
      }
    }
    if (!best) break;
    selected.push(best);
    best.covers.forEach((value) => uncovered.delete(value));
  }

  return { primes, essential, selected };
}

function latexExpression(expression: string): string {
  return expression
    .replace(/¬/g, '\\neg ')
    .replace(/∧/g, '\\land ')
    .replace(/∨/g, '\\lor ')
    .replace(/↔/g, '\\leftrightarrow ')
    .replace(/→/g, '\\to ')
    .replace(/⊕/g, '\\oplus ');
}

export function simplifyBoolean(source: string): BooleanResult {
  const tree = parse(source);
  const variablesSet = new Set<string>();
  collect(tree, variablesSet);
  const variables = [...variablesSet].sort();
  if (variables.length > 6) {
    throw new Error('Use at most six variables in the simplifier.');
  }

  const rows: BooleanRow[] = [];
  const minterms: number[] = [];
  const maxterms: number[] = [];
  const total = 1 << variables.length;
  for (let mask = 0; mask < total; mask += 1) {
    const values: Record<string, boolean> = {};
    variables.forEach((name, bit) => {
      values[name] = Boolean(mask & (1 << (variables.length - 1 - bit)));
    });
    const output = evaluate(tree, values);
    rows.push({ values: [...variables.map((name) => values[name]), output] });
    if (output) minterms.push(mask);
    else maxterms.push(mask);
  }

  const canonicalSop = minterms.length
    ? minterms
        .map((value) => {
          const bits = binary(value, variables.length);
          return bits
            .split('')
            .map((bit, index) => (bit === '1' ? variables[index] : `¬${variables[index]}`))
            .join(' ∧ ');
        })
        .join(' ∨ ')
    : '0';

  const canonicalPos = maxterms.length
    ? maxterms
        .map((value) => {
          const bits = binary(value, variables.length);
          const clause = bits
            .split('')
            .map((bit, index) => (bit === '1' ? `¬${variables[index]}` : variables[index]))
            .join(' ∨ ');
          return maxterms.length === 1 ? clause : `(${clause})`;
        })
        .join(' ∧ ')
    : '1';

  const { primes, essential, selected } = minimise(minterms, variables.length, variables);
  const minimal = selected.length ? selected.map((prime) => prime.expression).join(' ∨ ') : '0';

  const latex = buildLatex({
    source: label(tree),
    variables,
    minterms,
    maxterms,
    canonicalSop,
    canonicalPos,
    primes,
    essential,
    selected,
    minimal,
    rows
  });

  return {
    formula: source.trim(),
    variables,
    rows,
    minterms,
    maxterms,
    canonicalSop,
    canonicalPos,
    primes,
    essential,
    selected,
    minimal,
    latex
  };
}

function buildLatex(result: {
  source: string;
  variables: string[];
  minterms: number[];
  maxterms: number[];
  canonicalSop: string;
  canonicalPos: string;
  primes: PrimeImplicant[];
  essential: PrimeImplicant[];
  selected: PrimeImplicant[];
  minimal: string;
  rows: BooleanRow[];
}): string {
  const header = [...result.variables, 'F'].join(' & ');
  const body = result.rows
    .map((row) => row.values.map((value) => (value ? '1' : '0')).join(' & '))
    .join(' \\\\\n');
  const primes = result.primes
    .map((prime) => `${latexExpression(prime.expression)} & ${prime.covers.join(', ')}`)
    .join(' \\\\\n');

  return `\\documentclass[11pt]{article}
\\usepackage[a4paper,margin=2.2cm]{geometry}
\\usepackage{amsmath,amssymb}
\\setlength{\\parindent}{0pt}
\\begin{document}
\\section*{Boolean simplification}
\\[
${latexExpression(result.source)}
\\]
\\subsection*{Truth table}
\\[
\\begin{array}{${'c|'.repeat(result.variables.length)}c}
${header} \\\\ \\hline
${body}
\\end{array}
\\]
Minterms: $\\Sigma m(${result.minterms.join(', ') || '-'})$. Maxterms: $\\Pi M(${result.maxterms.join(', ') || '-'})$.

\\subsection*{Canonical forms}
SOP: \\[
${latexExpression(result.canonicalSop)}
\\]
POS: \\[
${latexExpression(result.canonicalPos)}
\\]

\\subsection*{Prime implicants}
\\[
\\begin{array}{l|l}
\\text{Term} & \\text{Minterms} \\\\ \\hline
${primes || '\\text{none}'}
\\end{array}
\\]
Essential prime implicants: ${result.essential.length ? result.essential.map((prime) => `$${latexExpression(prime.expression)}$`).join(', ') : 'none'}.

\\subsection*{Minimal sum of products}
\\[
${latexExpression(result.minimal)}
\\]
\\end{document}
`;
}
