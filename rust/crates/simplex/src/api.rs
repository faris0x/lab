use crate::engine::{self, Constraint, Model, Op, Sense};
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
    let model = parse_model(v)?;
    let solution = engine::solve(model)?;
    Ok(render::render(&solution))
}

fn parse_model(v: &Value) -> Result<Model, String> {
    let sense = match v.get("sense").and_then(Value::as_str) {
        Some("max") | Some("maximise") | Some("maximize") => Sense::Max,
        Some("min") | Some("minimise") | Some("minimize") => Sense::Min,
        _ => return Err("choose whether to maximise or minimise".into()),
    };

    let vars_value = v
        .get("vars")
        .and_then(Value::as_array)
        .ok_or("missing variables")?;
    if vars_value.is_empty() || vars_value.len() > 6 {
        return Err("use between 1 and 6 variables".into());
    }
    let mut vars: Vec<String> = Vec::with_capacity(vars_value.len());
    for item in vars_value {
        let name = item
            .as_str()
            .ok_or("variable names must be strings")?
            .trim()
            .to_string();
        if !valid_var_name(&name) {
            return Err(format!(
                "invalid variable name '{name}': use letters and digits, starting with a letter"
            ));
        }
        if vars.iter().any(|v| v == &name) {
            return Err(format!("duplicate variable name '{name}'"));
        }
        vars.push(name);
    }

    let obj_value = v
        .get("objective")
        .and_then(Value::as_array)
        .ok_or("missing objective coefficients")?;
    if obj_value.len() != vars.len() {
        return Err("the objective needs one coefficient per variable".into());
    }
    let mut objective = Vec::with_capacity(vars.len());
    for (j, c) in obj_value.iter().enumerate() {
        objective.push(
            rat_value(c).map_err(|e| format!("objective coefficient {}: {e}", j + 1))?,
        );
    }

    let cons_value = v
        .get("constraints")
        .and_then(Value::as_array)
        .ok_or("add at least one constraint")?;
    if cons_value.is_empty() || cons_value.len() > 10 {
        return Err("use between 1 and 10 constraints".into());
    }
    let mut constraints = Vec::with_capacity(cons_value.len());
    for (i, c) in cons_value.iter().enumerate() {
        let coeff_value = c
            .get("coeffs")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("constraint {} is missing coefficients", i + 1))?;
        if coeff_value.len() != vars.len() {
            return Err(format!(
                "constraint {} needs one coefficient per variable",
                i + 1
            ));
        }
        let mut coeffs = Vec::with_capacity(vars.len());
        for (j, x) in coeff_value.iter().enumerate() {
            coeffs.push(
                rat_value(x)
                    .map_err(|e| format!("constraint {} coefficient {}: {e}", i + 1, j + 1))?,
            );
        }
        let op = match c.get("op").and_then(Value::as_str) {
            Some("<=") | Some("le") => Op::Le,
            Some(">=") | Some("ge") => Op::Ge,
            Some("=") | Some("eq") => Op::Eq,
            _ => return Err(format!("constraint {} needs an inequality", i + 1)),
        };
        let rhs = rat_value(
            c.get("rhs")
                .ok_or_else(|| format!("constraint {} is missing a right-hand side", i + 1))?,
        )
        .map_err(|e| format!("constraint {} right-hand side: {e}", i + 1))?;
        constraints.push(Constraint { coeffs, op, rhs });
    }

    Ok(Model {
        sense,
        vars,
        objective,
        constraints,
    })
}

fn valid_var_name(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    s.len() <= 8 && chars.all(|c| c.is_ascii_alphanumeric())
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
            return Err("coefficients must be finite".into());
        }
        return Rat::parse(&f.to_string());
    }
    Err("coefficients must be numbers".into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn renders_a_full_solution() {
        let input = r#"{"sense":"min","vars":["x","y"],"objective":["2","3"],"constraints":[{"coeffs":["1","1"],"op":">=","rhs":"4"},{"coeffs":["1","3"],"op":">=","rhs":"6"}]}"#;
        let out = super::solve_json(input);
        let value: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value["ok"], true, "{out}");
        assert_eq!(value["status"], "optimal", "{out}");
        assert_eq!(value["optimum"]["plain"], "9", "{out}");
        assert!(!value["latex"]["document"].as_str().unwrap().is_empty());
        println!("{}", serde_json::to_string_pretty(&value).unwrap());
    }
}
