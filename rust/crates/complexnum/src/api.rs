use crate::engine::{arithmetic, polar_form, power, roots, Exact};
use crate::render;
use mathcore::Rat;
use serde_json::Value;

pub fn solve_json(input: &str) -> String {
    let value: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(e) => return error_json(&format!("invalid input: {e}")),
    };
    match run(&value) {
        Ok(out) => serde_json::to_string(&out).unwrap_or_else(|e| error_json(&e.to_string())),
        Err(e) => error_json(&e),
    }
}

fn error_json(message: &str) -> String {
    serde_json::json!({ "ok": false, "error": message }).to_string()
}

fn run(v: &Value) -> Result<render::Output, String> {
    let operation = v
        .get("operation")
        .and_then(Value::as_str)
        .ok_or("choose an operation")?;
    let z = parse_complex(v.get("z"), "z")?;
    let outcome = match operation {
        "arithmetic" => {
            let w = parse_complex(v.get("w"), "w")?;
            let op = v
                .get("op")
                .and_then(Value::as_str)
                .ok_or("choose an arithmetic operation")?;
            arithmetic(z, w, op)?
        }
        "polar" => polar_form(z),
        "power" => {
            let n = parse_n(v)?;
            power(z, n)?
        }
        "roots" => {
            let n = parse_n(v)?;
            roots(z, n)?
        }
        _ => return Err("choose arithmetic, polar, power or roots".into()),
    };
    Ok(render::render(&outcome))
}

fn parse_complex(value: Option<&Value>, name: &str) -> Result<Exact, String> {
    let parts = value
        .and_then(Value::as_array)
        .ok_or_else(|| format!("provide {name} as [real, imaginary]"))?;
    if parts.len() != 2 {
        return Err(format!("{name} needs a real and an imaginary part"));
    }
    Ok(Exact::new(
        rat_value(&parts[0]).map_err(|e| format!("{name} real part: {e}"))?,
        rat_value(&parts[1]).map_err(|e| format!("{name} imaginary part: {e}"))?,
    ))
}

fn parse_n(v: &Value) -> Result<i64, String> {
    v.get("n")
        .and_then(Value::as_i64)
        .ok_or_else(|| "choose an integer n".to_string())
}

fn rat_value(v: &Value) -> Result<Rat, String> {
    if let Some(s) = v.as_str() {
        if s.trim().is_empty() {
            return Err("enter a number".into());
        }
        return Rat::parse(s).map_err(|e| format!("{e} (got '{s}')"));
    }
    if let Some(i) = v.as_i64() {
        return Ok(Rat::int(i));
    }
    if let Some(f) = v.as_f64() {
        if !f.is_finite() {
            return Err("numbers must be finite".into());
        }
        return Rat::parse(&f.to_string());
    }
    Err("numbers must be numeric".into())
}
