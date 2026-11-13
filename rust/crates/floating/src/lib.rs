//! Number-base conversion, normalisation and custom/IEEE floating point.

use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{ToPrimitive, Zero};
use serde::Serialize;
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

const DIGITS: &[u8; 16] = b"0123456789ABCDEF";

#[derive(Clone, Debug, Serialize)]
pub struct BaseOutput {
    pub base: u32,
    pub text: String,
    pub exact: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct BaseResult {
    pub input: String,
    pub from: u32,
    pub negative: bool,
    pub outputs: Vec<BaseOutput>,
}

#[derive(Clone, Debug, Serialize)]
pub struct NormalisedResult {
    pub input: String,
    pub from: u32,
    pub sign: String,
    pub exponent: i32,
    pub fraction: String,
    pub binary: String,
    pub exact: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomFloatResult {
    pub bit_string: String,
    pub hex: String,
    pub sign: u32,
    pub exponent_field: u32,
    pub fraction: String,
    pub value: String,
    pub note: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IeeeResult {
    pub kind: String,
    pub bit_string: String,
    pub hex: String,
    pub sign: u32,
    pub exponent_field: u32,
    pub fraction: String,
    pub value: String,
    pub classification: String,
}

fn digit_value(character: char) -> Option<u32> {
    character.to_digit(16)
}

pub fn parse_rational(input: &str, base: u32) -> Result<(BigInt, BigUint), String> {
    let mut text = input.trim().replace(['_', ' '], "").to_ascii_lowercase();
    if text.is_empty() {
        return Err("Enter a value.".into());
    }
    let mut negative = false;
    if let Some(rest) = text.strip_prefix('-') {
        negative = true;
        text = rest.to_string();
    } else if let Some(rest) = text.strip_prefix('+') {
        text = rest.to_string();
    }
    if base == 16 {
        if let Some(rest) = text.strip_prefix("0x") {
            text = rest.to_string();
        }
    } else if base == 8 {
        if let Some(rest) = text.strip_prefix("0o") {
            text = rest.to_string();
        }
    } else if base == 2 {
        if let Some(rest) = text.strip_prefix("0b") {
            text = rest.to_string();
        }
    }

    let parts: Vec<&str> = text.split('.').collect();
    if parts.len() > 2 {
        return Err("Use at most one decimal point.".into());
    }
    let whole_text = if parts[0].is_empty() { "0" } else { parts[0] };
    let fraction_text = parts.get(1).copied().unwrap_or("");

    let base_big = BigUint::from(base);
    let mut whole = BigUint::zero();
    for character in whole_text.chars() {
        let digit =
            digit_value(character).ok_or_else(|| format!("Invalid digit “{character}”."))?;
        if digit >= base {
            return Err(format!("Digit {character} is not valid in base {base}."));
        }
        whole = whole * &base_big + BigUint::from(digit);
    }
    let mut fraction = BigUint::zero();
    for character in fraction_text.chars() {
        let digit =
            digit_value(character).ok_or_else(|| format!("Invalid digit “{character}”."))?;
        if digit >= base {
            return Err(format!("Digit {character} is not valid in base {base}."));
        }
        fraction = fraction * &base_big + BigUint::from(digit);
    }
    let den = base_big.pow(fraction_text.chars().count() as u32);
    let magnitude = whole * &den + fraction;
    let sign = if negative { Sign::Minus } else { Sign::Plus };
    Ok((BigInt::from_biguint(sign, magnitude), den))
}

fn compare_pow2(num: &BigUint, den: &BigUint, exponent: i32) -> Ordering {
    let (left, right) = if exponent >= 0 {
        (num.clone(), den << exponent as usize)
    } else {
        (num << (-exponent) as usize, den.clone())
    };
    left.cmp(&right)
}

fn ge_pow2(num: &BigUint, den: &BigUint, exponent: i32) -> bool {
    compare_pow2(num, den, exponent) != Ordering::Less
}

fn digit_text(value: u32) -> String {
    char::from(DIGITS[value as usize]).to_string()
}

pub fn to_base(
    num: &BigInt,
    den: &BigUint,
    base: u32,
    max_fraction: usize,
) -> Result<BaseOutput, String> {
    let negative = num.sign() == Sign::Minus;
    let abs = num.magnitude().clone();
    let whole = &abs / den;
    let mut remainder = &abs % den;

    let whole_text = if whole.is_zero() {
        "0".to_string()
    } else {
        whole.to_str_radix(base).to_uppercase()
    };
    let base_big = BigUint::from(base);
    let mut fraction_text = String::new();
    let mut exact = remainder.is_zero();
    let mut cycle_start: Option<usize> = None;
    let mut seen: HashMap<BigUint, usize> = HashMap::new();
    let mut index = 0usize;
    while !remainder.is_zero() && index < max_fraction {
        if let Some(start) = seen.get(&remainder) {
            cycle_start = Some(*start);
            break;
        }
        seen.insert(remainder.clone(), index);
        remainder *= &base_big;
        let digit = &remainder / den;
        fraction_text.push_str(&digit_text(digit.to_u32().unwrap_or(0)));
        remainder %= den;
        index += 1;
    }
    if remainder.is_zero() {
        exact = true;
    }

    let mut text = whole_text;
    if !fraction_text.is_empty() {
        if let Some(start) = cycle_start {
            text.push('.');
            text.push_str(&fraction_text[..start]);
            text.push('(');
            text.push_str(&fraction_text[start..]);
            text.push(')');
        } else if !exact {
            text.push('.');
            text.push_str(&fraction_text);
            text.push('…');
        } else {
            text.push('.');
            text.push_str(&fraction_text);
        }
    }

    Ok(BaseOutput {
        base,
        text: format!("{}{}", if negative { "-" } else { "" }, text),
        exact,
    })
}

pub fn convert_bases(input: &str, from: u32) -> Result<BaseResult, String> {
    let (num, den) = parse_rational(input, from)?;
    let mut outputs = Vec::new();
    for base in [2u32, 8, 10, 16] {
        outputs.push(to_base(&num, &den, base, 24)?);
    }
    Ok(BaseResult {
        input: input.trim().to_string(),
        from,
        negative: num.sign() == Sign::Minus,
        outputs,
    })
}

pub fn normalise(input: &str, from: u32) -> Result<NormalisedResult, String> {
    let (num, den) = parse_rational(input, from)?;
    if num.is_zero() {
        return Ok(NormalisedResult {
            input: input.trim().to_string(),
            from,
            sign: "0".into(),
            exponent: 0,
            fraction: String::new(),
            binary: "0".into(),
            exact: true,
        });
    }
    let negative = num.sign() == Sign::Minus;
    let abs = num.magnitude().clone();
    let mut exponent = 0i32;
    if ge_pow2(&abs, &den, 0) {
        while ge_pow2(&abs, &den, exponent + 1) {
            exponent += 1;
        }
    } else {
        while !ge_pow2(&abs, &den, exponent) {
            exponent -= 1;
        }
    }

    let mantissa_num = if exponent >= 0 {
        abs
    } else {
        abs << (-exponent) as usize
    };
    let mantissa_den = if exponent >= 0 {
        den.clone() << exponent as usize
    } else {
        den.clone()
    };
    let mut remainder = &mantissa_num - &mantissa_den;
    let mut fraction = String::new();
    let mut seen: HashMap<BigUint, usize> = HashMap::new();
    let mut index = 0usize;
    while !remainder.is_zero() && index < 32 {
        if seen.contains_key(&remainder) {
            break;
        }
        seen.insert(remainder.clone(), index);
        remainder <<= 1usize;
        if remainder >= mantissa_den {
            fraction.push('1');
            remainder -= &mantissa_den;
        } else {
            fraction.push('0');
        }
        index += 1;
    }
    let exact = remainder.is_zero();

    Ok(NormalisedResult {
        input: input.trim().to_string(),
        from,
        sign: if negative { "-".into() } else { String::new() },
        exponent,
        fraction: fraction.clone(),
        binary: format!(
            "{}1.{} × 2^{}",
            if negative { "-" } else { "" },
            if fraction.is_empty() { "0" } else { &fraction },
            exponent
        ),
        exact,
    })
}

fn group_hex(bits: &str) -> String {
    let padded = "0".repeat((4 - bits.len() % 4) % 4) + bits;
    let mut hex = String::new();
    for index in (0..padded.len()).step_by(4) {
        let value = u8::from_str_radix(&padded[index..index + 4], 2).unwrap_or(0);
        hex.push_str(&format!("{value:X}"));
    }
    hex
}

pub fn decimal(value: f64) -> String {
    if value.is_nan() {
        return "NaN".into();
    }
    if value.is_infinite() {
        return if value > 0.0 {
            "∞".into()
        } else {
            "-∞".into()
        };
    }
    if value == 0.0 {
        return "0".into();
    }
    let scientific = format!("{value:.9e}");
    let (mantissa, exponent_text) = scientific.split_once('e').unwrap_or(("0", "0"));
    let exponent: i32 = exponent_text.parse().unwrap_or(0);
    let negative = mantissa.starts_with('-');
    let mut digits: String = mantissa
        .chars()
        .filter(|character| character.is_ascii_digit())
        .collect();
    while digits.len() > 1 && digits.ends_with('0') {
        digits.pop();
    }
    let point = exponent + 1;
    let mut text = if point <= 0 {
        format!("0.{}{}", "0".repeat((-point) as usize), digits)
    } else if point as usize >= digits.len() {
        format!("{}{}", digits, "0".repeat(point as usize - digits.len()))
    } else {
        format!(
            "{}.{}",
            &digits[..point as usize],
            &digits[point as usize..]
        )
    };
    if negative {
        text.insert(0, '-');
    }
    text
}

fn pow2(exponent: i32) -> f64 {
    2f64.powi(exponent)
}

pub fn encode_custom(
    value_text: &str,
    exponent_bits: u32,
    mantissa_bits: u32,
) -> Result<CustomFloatResult, String> {
    if !(2..=8).contains(&exponent_bits) {
        return Err("Use between 2 and 8 exponent bits.".into());
    }
    if !(2..=24).contains(&mantissa_bits) {
        return Err("Use between 2 and 24 mantissa bits.".into());
    }
    let (num, _) = parse_rational(value_text, 10)?;
    let sign = if num.sign() == Sign::Minus { 1 } else { 0 };
    if num.is_zero() {
        let bits = "0".repeat((exponent_bits + mantissa_bits + 1) as usize);
        return Ok(CustomFloatResult {
            hex: group_hex(&bits),
            bit_string: bits,
            sign: 0,
            exponent_field: 0,
            fraction: "0".repeat(mantissa_bits as usize),
            value: "0".into(),
            note: "Zero is stored as all zero bits.".into(),
        });
    }
    let normal = normalise(value_text, 10)?;
    let bias = 2i32.pow(exponent_bits - 1) - 1;
    let max_exponent = 2u32.pow(exponent_bits) - 1;
    let exponent_field = normal.exponent + bias;
    if exponent_field <= 0 {
        return Err("The exponent underflows this format.".into());
    }
    if exponent_field >= max_exponent as i32 {
        return Err("The exponent overflows this format.".into());
    }
    let mut fraction: String = normal
        .fraction
        .chars()
        .take(mantissa_bits as usize)
        .collect();
    while fraction.len() < mantissa_bits as usize {
        fraction.push('0');
    }
    let exponent_binary = format!("{exponent_field:0width$b}", width = exponent_bits as usize);
    let bit_string = format!("{sign}{exponent_binary}{fraction}");
    let note = if normal.fraction.len() > mantissa_bits as usize {
        format!("The mantissa was truncated to {mantissa_bits} bits.")
    } else if normal.exact {
        "The value is represented exactly.".into()
    } else {
        "The fraction terminates after the stored mantissa bits.".into()
    };
    let fraction_value = u64::from_str_radix(if fraction.is_empty() { "0" } else { &fraction }, 2)
        .unwrap_or(0) as f64
        / pow2(mantissa_bits as i32);
    let magnitude = (1.0 + fraction_value) * pow2(exponent_field - bias);
    Ok(CustomFloatResult {
        bit_string: bit_string.clone(),
        hex: group_hex(&bit_string),
        sign,
        exponent_field: exponent_field as u32,
        fraction,
        value: decimal(if sign == 1 { -magnitude } else { magnitude }),
        note,
    })
}

pub fn decode_custom(
    bit_string: &str,
    exponent_bits: u32,
    mantissa_bits: u32,
) -> Result<CustomFloatResult, String> {
    let total = (exponent_bits + mantissa_bits + 1) as usize;
    let mut bits = bit_string.trim().replace(['_', ' '], "");
    if bits.to_ascii_lowercase().starts_with("0x") {
        bits = bits[2..].to_string();
    }
    let is_hex = !bits.is_empty() && bits.chars().all(|character| character.is_ascii_hexdigit());
    if is_hex && bits.len() <= total.div_ceil(4) {
        bits = bits
            .chars()
            .map(|character| format!("{:04b}", character.to_digit(16).unwrap_or(0)))
            .collect();
        if bits.len() > total {
            bits = bits[bits.len() - total..].to_string();
        }
    }
    if bits.is_empty()
        || !bits
            .chars()
            .all(|character| character == '0' || character == '1')
    {
        return Err("Enter a binary or hexadecimal bit pattern.".into());
    }
    if bits.len() != total {
        return Err(format!("This format needs {total} bits."));
    }
    let chars: Vec<char> = bits.chars().collect();
    let sign = chars[0].to_digit(2).unwrap_or(0);
    let exponent_field = u32::from_str_radix(&bits[1..1 + exponent_bits as usize], 2).unwrap_or(0);
    let fraction = bits[1 + exponent_bits as usize..].to_string();
    let bias = 2i32.pow(exponent_bits - 1) - 1;
    let max_exponent = 2u32.pow(exponent_bits) - 1;
    let (value, note) = if exponent_field == max_exponent {
        let value = if fraction.chars().all(|character| character == '0') {
            if sign == 1 {
                "-∞".to_string()
            } else {
                "∞".to_string()
            }
        } else {
            "NaN".to_string()
        };
        (
            value,
            "The reserved exponent marks infinity or NaN.".to_string(),
        )
    } else if exponent_field == 0 {
        let mantissa = u64::from_str_radix(if fraction.is_empty() { "0" } else { &fraction }, 2)
            .unwrap_or(0) as f64
            / pow2(mantissa_bits as i32);
        (
            decimal(if sign == 1 { -1.0 } else { 1.0 } * mantissa * pow2(1 - bias)),
            "Subnormal: the leading bit is 0 and the exponent is 1 − bias.".to_string(),
        )
    } else {
        let mantissa = u64::from_str_radix(if fraction.is_empty() { "0" } else { &fraction }, 2)
            .unwrap_or(0) as f64
            / pow2(mantissa_bits as i32);
        (
            decimal(
                if sign == 1 { -1.0 } else { 1.0 }
                    * (1.0 + mantissa)
                    * pow2(exponent_field as i32 - bias),
            ),
            format!(
                "Exponent {exponent_field} − bias {bias} = {}.",
                exponent_field as i32 - bias
            ),
        )
    };
    Ok(CustomFloatResult {
        bit_string: bits.clone(),
        hex: group_hex(&bits),
        sign,
        exponent_field,
        fraction,
        value,
        note,
    })
}

fn classify(exponent_field: u32, fraction: &str, max_exponent: u32) -> &'static str {
    if exponent_field == max_exponent {
        if fraction.chars().all(|character| character == '0') {
            "Infinity"
        } else {
            "NaN"
        }
    } else if exponent_field == 0 {
        if fraction.chars().all(|character| character == '0') {
            "Zero"
        } else {
            "Subnormal"
        }
    } else {
        "Normal"
    }
}

pub fn encode_ieee(value_text: &str, kind: &str) -> Result<IeeeResult, String> {
    let value: f64 = value_text
        .trim()
        .parse()
        .map_err(|_| "Enter a finite decimal number.".to_string())?;
    if !value.is_finite() {
        return Err("Enter a finite decimal number.".into());
    }
    let (bits, total_bits, exponent_bits, fraction_bits) = if kind == "single" {
        ((value as f32).to_bits() as u64, 32u32, 8u32, 23u32)
    } else {
        (value.to_bits(), 64u32, 11u32, 52u32)
    };
    let bit_string = format!("{bits:0width$b}", width = total_bits as usize);
    let sign = ((bits >> (total_bits - 1)) & 1) as u32;
    let exponent_field = ((bits >> fraction_bits) & ((1u64 << exponent_bits) - 1)) as u32;
    let fraction = format!(
        "{:0width$b}",
        bits & ((1u64 << fraction_bits) - 1),
        width = fraction_bits as usize
    );
    let max_exponent = (1u32 << exponent_bits) - 1;
    let classification = classify(exponent_field, &fraction, max_exponent).to_string();
    Ok(IeeeResult {
        kind: kind.to_string(),
        hex: group_hex(&bit_string).to_uppercase(),
        bit_string,
        sign,
        exponent_field,
        fraction,
        value: decimal(value),
        classification,
    })
}

pub fn decode_ieee(hex: &str, kind: &str) -> Result<IeeeResult, String> {
    let clean = hex.trim().replace(['_', ' '], "").to_ascii_lowercase();
    let clean = clean.trim_start_matches("0x").to_ascii_uppercase();
    let digits = if kind == "single" { 8 } else { 16 };
    if clean.len() != digits || !clean.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(format!("Enter exactly {digits} hexadecimal digits."));
    }
    let value =
        u64::from_str_radix(&clean, 16).map_err(|_| "Invalid hexadecimal digits.".to_string())?;
    let exponent_bits = if kind == "single" { 8u32 } else { 11u32 };
    let bits = format!("{value:0width$b}", width = digits * 4);
    let sign = bits.chars().next().unwrap_or('0').to_digit(2).unwrap_or(0);
    let exponent_field = u32::from_str_radix(&bits[1..1 + exponent_bits as usize], 2).unwrap_or(0);
    let fraction = bits[1 + exponent_bits as usize..].to_string();
    let max_exponent = (1u32 << exponent_bits) - 1;
    let classification = classify(exponent_field, &fraction, max_exponent).to_string();
    let decoded = if kind == "single" {
        f32::from_bits(value as u32) as f64
    } else {
        f64::from_bits(value)
    };
    Ok(IeeeResult {
        kind: kind.to_string(),
        bit_string: bits,
        hex: clean,
        sign,
        exponent_field,
        fraction,
        value: decimal(decoded),
        classification,
    })
}

pub fn solve_json(input: &str) -> String {
    let value: Value = match serde_json::from_str(input) {
        Ok(value) => value,
        Err(error) => {
            return serde_json::json!({ "ok": false, "error": format!("invalid input: {error}") })
                .to_string()
        }
    };
    match run(&value) {
        Ok(result) => serde_json::json!({ "ok": true, "result": result }).to_string(),
        Err(error) => serde_json::json!({ "ok": false, "error": error }).to_string(),
    }
}

fn run(value: &Value) -> Result<Value, String> {
    let operation = value
        .get("operation")
        .and_then(Value::as_str)
        .ok_or("choose an operation")?;
    let input = value
        .get("input")
        .and_then(Value::as_str)
        .ok_or("enter a value")?;
    let from = value.get("from").and_then(Value::as_u64).unwrap_or(10) as u32;
    let result = match operation {
        "convert" => {
            serde_json::to_value(convert_bases(input, from)?).map_err(|error| error.to_string())?
        }
        "normalise" => {
            serde_json::to_value(normalise(input, from)?).map_err(|error| error.to_string())?
        }
        "encodeCustom" => {
            let exponent_bits = value
                .get("exponentBits")
                .and_then(Value::as_u64)
                .ok_or("choose the exponent width")? as u32;
            let mantissa_bits = value
                .get("mantissaBits")
                .and_then(Value::as_u64)
                .ok_or("choose the mantissa width")? as u32;
            serde_json::to_value(encode_custom(input, exponent_bits, mantissa_bits)?)
                .map_err(|error| error.to_string())?
        }
        "decodeCustom" => {
            let exponent_bits = value
                .get("exponentBits")
                .and_then(Value::as_u64)
                .ok_or("choose the exponent width")? as u32;
            let mantissa_bits = value
                .get("mantissaBits")
                .and_then(Value::as_u64)
                .ok_or("choose the mantissa width")? as u32;
            serde_json::to_value(decode_custom(input, exponent_bits, mantissa_bits)?)
                .map_err(|error| error.to_string())?
        }
        "encodeIeee" => {
            let kind = value
                .get("kind")
                .and_then(Value::as_str)
                .ok_or("choose single or double precision")?;
            serde_json::to_value(encode_ieee(input, kind)?).map_err(|error| error.to_string())?
        }
        "decodeIeee" => {
            let kind = value
                .get("kind")
                .and_then(Value::as_str)
                .ok_or("choose single or double precision")?;
            serde_json::to_value(decode_ieee(input, kind)?).map_err(|error| error.to_string())?
        }
        _ => return Err("choose a floating-point operation".into()),
    };
    Ok(result)
}

#[wasm_bindgen]
pub fn solve(input: &str) -> String {
    solve_json(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_integers_and_fractions() {
        let (num, den) = parse_rational("FF", 16).unwrap();
        assert_eq!(num.to_string(), "255");
        assert_eq!(den.to_string(), "1");
        let (num, den) = parse_rational("10.1", 2).unwrap();
        assert_eq!(num.to_string(), "5");
        assert_eq!(den.to_string(), "2");
    }

    #[test]
    fn rejects_invalid_digits() {
        assert!(parse_rational("2", 2)
            .unwrap_err()
            .contains("not valid in base 2"));
    }

    #[test]
    fn writes_all_four_bases() {
        let result = convert_bases("255", 10).unwrap();
        let by_base: HashMap<u32, String> = result
            .outputs
            .iter()
            .map(|output| (output.base, output.text.clone()))
            .collect();
        assert_eq!(by_base[&2], "11111111");
        assert_eq!(by_base[&8], "377");
        assert_eq!(by_base[&10], "255");
        assert_eq!(by_base[&16], "FF");
    }

    #[test]
    fn normalises_binary_fraction() {
        let result = normalise("101.1", 2).unwrap();
        assert_eq!(result.exponent, 2);
        assert_eq!(result.fraction, "011");
        assert!(result.exact);
    }

    #[test]
    fn encodes_one_as_single() {
        let result = encode_ieee("1", "single").unwrap();
        assert_eq!(result.hex, "3F800000");
        assert_eq!(result.classification, "Normal");
        assert_eq!(result.bit_string.len(), 32);
    }

    #[test]
    fn decodes_one() {
        let result = decode_ieee("3F800000", "single").unwrap();
        assert_eq!(result.value, "1");
        assert_eq!(result.classification, "Normal");
    }

    #[test]
    fn rejects_non_finite_input() {
        assert!(encode_ieee("Infinity", "single")
            .unwrap_err()
            .contains("finite"));
    }
}
