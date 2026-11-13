use crate::render::{expand_series, MAX_ORDER};
use serde_json::Value;

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

fn run(value: &Value) -> Result<crate::render::SeriesResult, String> {
    let source = value
        .get("source")
        .and_then(Value::as_str)
        .ok_or("enter a function of x")?;
    let center = value
        .get("center")
        .and_then(Value::as_str)
        .ok_or("enter a centre")?;
    let order = value
        .get("order")
        .and_then(Value::as_i64)
        .ok_or("choose a whole-number degree")?;
    if order < 0 {
        return Err(format!(
            "Choose a whole number of terms between 0 and {MAX_ORDER}."
        ));
    }
    expand_series(source, center, order as usize)
}
