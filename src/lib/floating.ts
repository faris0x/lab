import { preloadSolver, solveJson } from './wasm';

export type Base = 2 | 8 | 10 | 16;

export interface BaseOutput {
  base: Base;
  text: string;
  exact: boolean;
}

export interface BaseResult {
  input: string;
  from: Base;
  negative: boolean;
  outputs: BaseOutput[];
}

export interface NormalisedResult {
  input: string;
  from: Base;
  sign: string;
  exponent: number;
  fraction: string;
  binary: string;
  exact: boolean;
}

export interface CustomFloatResult {
  bitString: string;
  hex: string;
  sign: number;
  exponentField: number;
  fraction: string;
  value: string;
  note: string;
}

export interface IeeeResult {
  kind: 'single' | 'double';
  bitString: string;
  hex: string;
  sign: number;
  exponentField: number;
  fraction: string;
  value: string;
  classification: string;
}

interface FloatingError {
  ok: false;
  error: string;
}

interface FloatingSuccess<T> {
  ok: true;
  result: T;
}

async function run<T>(input: Record<string, unknown>): Promise<T> {
  const output = await solveJson<FloatingSuccess<T> | FloatingError>(
    'floating',
    input
  );
  if (!output.ok) {
    throw new Error(output.error);
  }
  return output.result;
}

export function preloadFloating(): void {
  preloadSolver('floating');
}

export function convertBases(input: string, from: Base): Promise<BaseResult> {
  return run<BaseResult>({ operation: 'convert', input, from });
}

export function normalise(input: string, from: Base): Promise<NormalisedResult> {
  return run<NormalisedResult>({ operation: 'normalise', input, from });
}

export function encodeCustom(
  value: string,
  exponentBits: number,
  mantissaBits: number
): Promise<CustomFloatResult> {
  return run<CustomFloatResult>({
    operation: 'encodeCustom',
    input: value,
    exponentBits,
    mantissaBits
  });
}

export function decodeCustom(
  bitString: string,
  exponentBits: number,
  mantissaBits: number
): Promise<CustomFloatResult> {
  return run<CustomFloatResult>({
    operation: 'decodeCustom',
    input: bitString,
    exponentBits,
    mantissaBits
  });
}

export function encodeIeee(
  value: string,
  kind: 'single' | 'double'
): Promise<IeeeResult> {
  return run<IeeeResult>({ operation: 'encodeIeee', input: value, kind });
}

export function decodeIeee(
  hex: string,
  kind: 'single' | 'double'
): Promise<IeeeResult> {
  return run<IeeeResult>({ operation: 'decodeIeee', input: hex, kind });
}
