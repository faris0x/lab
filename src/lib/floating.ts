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

const DIGITS = '0123456789ABCDEF';

function digitValue(character: string): number {
  const value = DIGITS.indexOf(character.toUpperCase());
  if (value < 0) throw new Error(`Invalid digit “${character}”.`);
  return value;
}

export function parseRational(input: string, base: Base): { num: bigint; den: bigint } {
  let text = input.trim().replace(/[_\s]/g, '').toLowerCase();
  if (!text) throw new Error('Enter a value.');
  let negative = false;
  if (text.startsWith('-')) {
    negative = true;
    text = text.slice(1);
  } else if (text.startsWith('+')) {
    text = text.slice(1);
  }
  if (base === 16 && text.startsWith('0x')) text = text.slice(2);
  if (base === 8 && text.startsWith('0o')) text = text.slice(2);
  if (base === 2 && text.startsWith('0b')) text = text.slice(2);

  const parts = text.split('.');
  if (parts.length > 2) throw new Error('Use at most one decimal point.');
  const wholeText = parts[0] || '0';
  const fractionText = parts[1] || '';

  let whole = 0n;
  for (const character of wholeText) {
    const digit = digitValue(character);
    if (digit >= base) throw new Error(`Digit ${character} is not valid in base ${base}.`);
    whole = whole * BigInt(base) + BigInt(digit);
  }
  let fraction = 0n;
  for (const character of fractionText) {
    const digit = digitValue(character);
    if (digit >= base) throw new Error(`Digit ${character} is not valid in base ${base}.`);
    fraction = fraction * BigInt(base) + BigInt(digit);
  }
  const den = BigInt(base) ** BigInt(fractionText.length);
  const num = whole * den + fraction;
  return { num: negative ? -num : num, den };
}

function comparePow2(num: bigint, den: bigint, exponent: number): number {
  const abs = num < 0n ? -num : num;
  const left = exponent >= 0 ? abs : abs << BigInt(-exponent);
  const right = exponent >= 0 ? den << BigInt(exponent) : den;
  return left === right ? 0 : left < right ? -1 : 1;
}

function gePow2(num: bigint, den: bigint, exponent: number): boolean {
  return comparePow2(num, den, exponent) >= 0;
}

export function toBase(num: bigint, den: bigint, base: Base, maxFraction = 24): BaseOutput {
  const negative = num < 0n;
  let abs = negative ? -num : num;
  let whole = abs / den;
  let remainder = abs % den;

  let wholeText = '';
  if (whole === 0n) wholeText = '0';
  while (whole > 0n) {
    wholeText = DIGITS[Number(whole % BigInt(base))] + wholeText;
    whole /= BigInt(base);
  }

  let fractionText = '';
  let exact = remainder === 0n;
  let cycleStart = -1;
  const seen = new Map<string, number>();
  let index = 0;
  while (remainder !== 0n && index < maxFraction) {
    const key = remainder.toString();
    if (seen.has(key)) {
      cycleStart = seen.get(key) ?? -1;
      break;
    }
    seen.set(key, index);
    remainder *= BigInt(base);
    fractionText += DIGITS[Number(remainder / den)];
    remainder %= den;
    index += 1;
  }
  if (remainder === 0n) exact = true;

  let text = wholeText;
  if (fractionText) {
    if (cycleStart >= 0) {
      text += `.${fractionText.slice(0, cycleStart)}(${fractionText.slice(cycleStart)})`;
    } else if (!exact) {
      text += `.${fractionText}…`;
    } else {
      text += `.${fractionText}`;
    }
  }
  return { base, text: `${negative ? '-' : ''}${text}`, exact };
}

export function convertBases(input: string, from: Base): BaseResult {
  const { num, den } = parseRational(input, from);
  const bases: Base[] = [2, 8, 10, 16];
  return {
    input: input.trim(),
    from,
    negative: num < 0n,
    outputs: bases.map((base) => toBase(num, den, base))
  };
}

export function normalise(input: string, from: Base): NormalisedResult {
  const { num, den } = parseRational(input, from);
  if (num === 0n) {
    return { input: input.trim(), from, sign: '0', exponent: 0, fraction: '', binary: '0', exact: true };
  }
  const negative = num < 0n;
  const abs = negative ? -num : num;
  let exponent = 0;
  if (gePow2(abs, den, 0)) {
    while (gePow2(abs, den, exponent + 1)) exponent += 1;
  } else {
    while (!gePow2(abs, den, exponent)) exponent -= 1;
  }

  const mantissaNum = exponent >= 0 ? abs : abs << BigInt(-exponent);
  const mantissaDen = exponent >= 0 ? den << BigInt(exponent) : den;
  let remainder = mantissaNum - mantissaDen;
  let fraction = '';
  const seen = new Map<string, number>();
  let index = 0;
  while (remainder !== 0n && index < 32) {
    const key = remainder.toString();
    if (seen.has(key)) break;
    seen.set(key, index);
    remainder *= 2n;
    fraction += remainder / mantissaDen === 1n ? '1' : '0';
    remainder %= mantissaDen;
    index += 1;
  }
  const exact = remainder === 0n;

  return {
    input: input.trim(),
    from,
    sign: negative ? '-' : '',
    exponent,
    fraction,
    binary: `${negative ? '-' : ''}1.${fraction || '0'} × 2^${exponent}`,
    exact
  };
}

function groupHex(bits: string): string {
  const padded = bits.padStart(Math.ceil(bits.length / 4) * 4, '0');
  let hex = '';
  for (let i = 0; i < padded.length; i += 4) {
    hex += Number.parseInt(padded.slice(i, i + 4), 2).toString(16).toUpperCase();
  }
  return hex;
}

function decimal(value: number): string {
  if (Number.isNaN(value)) return 'NaN';
  if (!Number.isFinite(value)) return value > 0 ? '∞' : '-∞';
  return Number(value.toPrecision(10)).toString();
}

export function encodeCustom(valueText: string, exponentBits: number, mantissaBits: number): CustomFloatResult {
  if (exponentBits < 2 || exponentBits > 8) throw new Error('Use between 2 and 8 exponent bits.');
  if (mantissaBits < 2 || mantissaBits > 24) throw new Error('Use between 2 and 24 mantissa bits.');
  const { num, den } = parseRational(valueText, 10);
  const sign = num < 0n ? 1 : 0;
  if (num === 0n) {
    const bits = '0'.repeat(exponentBits + mantissaBits + 1);
    return {
      bitString: bits,
      hex: groupHex(bits),
      sign: 0,
      exponentField: 0,
      fraction: '0'.repeat(mantissaBits),
      value: '0',
      note: 'Zero is stored as all zero bits.'
    };
  }
  const normal = normalise(valueText, 10);
  const bias = 2 ** (exponentBits - 1) - 1;
  const maxExponent = 2 ** exponentBits - 1;
  const exponentField = normal.exponent + bias;
  if (exponentField <= 0) throw new Error('The exponent underflows this format.');
  if (exponentField >= maxExponent) throw new Error('The exponent overflows this format.');
  const fraction = normal.fraction.slice(0, mantissaBits).padEnd(mantissaBits, '0');
  const bitString =
    String(sign) +
    exponentField.toString(2).padStart(exponentBits, '0') +
    fraction;
  const note = normal.fraction.length > mantissaBits
    ? `The mantissa was truncated to ${mantissaBits} bits.`
    : normal.exact
      ? 'The value is represented exactly.'
      : 'The fraction terminates after the stored mantissa bits.';
  const value = (1 + Number.parseInt(fraction || '0', 2) / 2 ** mantissaBits) * 2 ** (exponentField - bias);
  return {
    bitString,
    hex: groupHex(bitString),
    sign,
    exponentField,
    fraction,
    value: decimal(sign ? -value : value),
    note
  };
}

export function decodeCustom(bitString: string, exponentBits: number, mantissaBits: number): CustomFloatResult {
  let bits = bitString.trim().replace(/[_\s]/g, '').replace(/^0x/i, '');
  if (/^[0-9a-f]+$/i.test(bits) && bits.length <= Math.ceil((exponentBits + mantissaBits + 1) / 4)) {
    bits = [...bits]
      .map((character) => Number.parseInt(character, 16).toString(2).padStart(4, '0'))
      .join('')
      .slice(-(exponentBits + mantissaBits + 1));
  }
  if (!/^[01]+$/.test(bits)) throw new Error('Enter a binary or hexadecimal bit pattern.');
  if (bits.length !== exponentBits + mantissaBits + 1) {
    throw new Error(`This format needs ${exponentBits + mantissaBits + 1} bits.`);
  }
  const sign = Number(bits[0]);
  const exponentField = Number.parseInt(bits.slice(1, 1 + exponentBits), 2);
  const fraction = bits.slice(1 + exponentBits);
  const bias = 2 ** (exponentBits - 1) - 1;
  const maxExponent = 2 ** exponentBits - 1;
  let value: string;
  let note: string;
  if (exponentField === maxExponent) {
    value = fraction === '0'.repeat(mantissaBits) ? (sign ? '-∞' : '∞') : 'NaN';
    note = 'The reserved exponent marks infinity or NaN.';
  } else if (exponentField === 0) {
    const mantissa = Number.parseInt(fraction || '0', 2) / 2 ** mantissaBits;
    value = decimal((sign ? -1 : 1) * mantissa * 2 ** (1 - bias));
    note = 'Subnormal: the leading bit is 0 and the exponent is 1 − bias.';
  } else {
    const mantissa = Number.parseInt(fraction || '0', 2) / 2 ** mantissaBits;
    value = decimal((sign ? -1 : 1) * (1 + mantissa) * 2 ** (exponentField - bias));
    note = `Exponent ${exponentField} − bias ${bias} = ${exponentField - bias}.`;
  }
  return {
    bitString: bits,
    hex: groupHex(bits),
    sign,
    exponentField,
    fraction,
    value,
    note
  };
}

export function encodeIeee(valueText: string, kind: 'single' | 'double'): IeeeResult {
  const value = Number.parseFloat(valueText);
  if (!Number.isFinite(value)) throw new Error('Enter a finite decimal number.');
  const buffer = new ArrayBuffer(kind === 'single' ? 4 : 8);
  const view = new DataView(buffer);
  if (kind === 'single') view.setFloat32(0, value, false);
  else view.setFloat64(0, value, false);
  const exponentBits = kind === 'single' ? 8 : 11;
  const fractionBits = kind === 'single' ? 23 : 52;
  const total = exponentBits + fractionBits + 1;
  let bits = '';
  for (let i = 0; i < (kind === 'single' ? 4 : 8); i += 1) {
    bits += view.getUint8(i).toString(2).padStart(8, '0');
  }
  const sign = Number(bits[0]);
  const exponentField = Number.parseInt(bits.slice(1, 1 + exponentBits), 2);
  const fraction = bits.slice(1 + exponentBits);
  const bias = 2 ** (exponentBits - 1) - 1;
  const max = 2 ** exponentBits - 1;
  const classification =
    exponentField === max
      ? fraction === '0'.repeat(fractionBits)
        ? 'Infinity'
        : 'NaN'
      : exponentField === 0
        ? fraction === '0'.repeat(fractionBits)
          ? 'Zero'
          : 'Subnormal'
        : 'Normal';
  return {
    kind,
    bitString: bits,
    hex: groupHex(bits).padStart(total / 4, '0'),
    sign,
    exponentField,
    fraction,
    value: decimal(value),
    classification
  };
}

export function decodeIeee(hex: string, kind: 'single' | 'double'): IeeeResult {
  const clean = hex.trim().replace(/[_\s]/g, '').replace(/^0x/i, '');
  const digits = kind === 'single' ? 8 : 16;
  if (!new RegExp(`^[0-9a-fA-F]{${digits}}$`).test(clean)) {
    throw new Error(`Enter exactly ${digits} hexadecimal digits.`);
  }
  const buffer = new ArrayBuffer(kind === 'single' ? 4 : 8);
  const view = new DataView(buffer);
  if (kind === 'single') view.setUint32(0, Number.parseInt(clean, 16), false);
  else view.setBigUint64(0, BigInt(`0x${clean}`), false);
  const value = kind === 'single' ? view.getFloat32(0, false) : view.getFloat64(0, false);
  const exponentBits = kind === 'single' ? 8 : 11;
  const fractionBits = kind === 'single' ? 23 : 52;
  const bits = [...clean]
    .map((character) => Number.parseInt(character, 16).toString(2).padStart(4, '0'))
    .join('');
  const sign = Number(bits[0]);
  const exponentField = Number.parseInt(bits.slice(1, 1 + exponentBits), 2);
  const fraction = bits.slice(1 + exponentBits);
  const max = 2 ** exponentBits - 1;
  const classification =
    exponentField === max
      ? fraction === '0'.repeat(fractionBits)
        ? 'Infinity'
        : 'NaN'
      : exponentField === 0
        ? fraction === '0'.repeat(fractionBits)
          ? 'Zero'
          : 'Subnormal'
        : 'Normal';
  return {
    kind,
    bitString: bits,
    hex: clean.toUpperCase(),
    sign,
    exponentField,
    fraction,
    value: decimal(value),
    classification
  };
}
