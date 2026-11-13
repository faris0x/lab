import { preloadSolver, solveJson } from './wasm';

export interface SeriesStep {
  k: number;
  derivativeLatex: string;
  valueLatex: string;
  coefficientLatex: string;
  coefficientPlain: string;
  termLatex: string;
}

export interface GeneralTermResult {
  termLatex: string;
  termPlain: string;
  sumLatex: string;
  note: string;
}

export interface SeriesResult {
  source: string;
  functionLatex: string;
  center: string;
  centerLatex: string;
  order: number;
  kind: 'maclaurin' | 'taylor';
  seriesLatex: string;
  steps: SeriesStep[];
  summary: string[];
  summaryLatex: string[];
  latex: string;
  general: GeneralTermResult | null;
}

interface SeriesError {
  ok: false;
  error: string;
}

interface SeriesSuccess {
  ok: true;
  result: SeriesResult;
}

export const MAX_ORDER = 12;

export function preloadSeries(): void {
  preloadSolver('series');
}

export async function expandSeries(
  source: string,
  center: string,
  order: number
): Promise<SeriesResult> {
  const output = await solveJson<SeriesSuccess | SeriesError>('series', {
    source,
    center,
    order
  });
  if (!output.ok) {
    throw new Error(output.error);
  }
  return output.result;
}
