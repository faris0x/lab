// Copyright (c) 2026 Faris Alfarhan
// SPDX-License-Identifier: GPL-3.0-only

use crate::engine::{
    self, Constraint, FinishOptions, Header, Method, Model, Op, Sense, Tableau, VarKind,
};
use crate::render;
use mathcore::{MVal, Rat};
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
    let mode = v.get("mode").and_then(Value::as_str).unwrap_or("model");
    let solution = match mode {
        "model" => {
            let method = parse_method(v)?;
            engine::solve_with_method(parse_model(v)?, method)?
        }
        "tableau" => {
            let (tableau, options) = parse_tableau(v)?;
            engine::finish(tableau, options)?
        }
        other => return Err(format!("unknown input mode '{other}'")),
    };
    Ok(render::render(&solution))
}

fn parse_tableau(v: &Value) -> Result<(Tableau, FinishOptions), String> {
    let header_values = v
        .get("headers")
        .and_then(Value::as_array)
        .ok_or("provide tableau column names")?;
    if header_values.is_empty() || header_values.len() > 16 {
        return Err("use between 1 and 16 tableau columns".into());
    }
    let mut headers = Vec::with_capacity(header_values.len());
    for item in header_values {
        let name = item
            .as_str()
            .ok_or("tableau column names must be strings")?
            .trim()
            .to_string();
        if name.is_empty() {
            return Err("tableau column names cannot be empty".into());
        }
        headers.push(Header {
            name,
            kind: VarKind::Decision,
        });
    }
    let width = headers.len();

    let artificial_cols =
        parse_index_list(v.get("artificialColumns"), "artificial columns", width)?;
    for &column in &artificial_cols {
        headers[column].kind = VarKind::Artificial;
    }
    let decision_cols = match v.get("decisionColumns") {
        Some(_) => parse_index_list(v.get("decisionColumns"), "decision columns", width)?,
        None => (0..width)
            .filter(|column| !artificial_cols.contains(column))
            .collect(),
    };

    let row_values = v
        .get("rows")
        .and_then(Value::as_array)
        .ok_or("provide the tableau rows")?;
    if row_values.is_empty() || row_values.len() > 10 {
        return Err("use between 1 and 10 tableau rows".into());
    }
    let mut rows = Vec::with_capacity(row_values.len());
    for (r, row_value) in row_values.iter().enumerate() {
        let cells = row_value
            .as_array()
            .ok_or_else(|| format!("row {} must be an array", r + 1))?;
        if cells.len() != width {
            return Err(format!("row {} needs exactly {width} values", r + 1));
        }
        let mut row = Vec::with_capacity(width);
        for (c, cell) in cells.iter().enumerate() {
            row.push(
                parse_mval(cell).map_err(|e| format!("row {}, column {}: {e}", r + 1, c + 1))?,
            );
        }
        rows.push(row);
    }

    let rhs_values = v
        .get("rhs")
        .and_then(Value::as_array)
        .ok_or("provide the right-hand side")?;
    if rhs_values.len() != rows.len() {
        return Err("the right-hand side needs one value per row".into());
    }
    let mut rhs = Vec::with_capacity(rhs_values.len());
    for (i, value) in rhs_values.iter().enumerate() {
        rhs.push(parse_mval(value).map_err(|e| format!("right-hand side {}: {e}", i + 1))?);
    }

    let obj_values = v
        .get("obj")
        .and_then(Value::as_array)
        .ok_or("provide the objective row")?;
    if obj_values.len() != width {
        return Err("the objective row needs one value per column".into());
    }
    let mut obj = Vec::with_capacity(width);
    for (i, value) in obj_values.iter().enumerate() {
        obj.push(parse_mval(value).map_err(|e| format!("objective column {}: {e}", i + 1))?);
    }
    let obj_rhs = parse_mval(
        v.get("objRhs")
            .ok_or("provide the objective right-hand side")?,
    )
    .map_err(|e| format!("objective right-hand side: {e}"))?;

    let basis = parse_index_list(v.get("basis"), "basis", width)?;
    if basis.len() != rows.len() {
        return Err("the basis needs one variable per row".into());
    }

    let report_as = v.get("reportAs").and_then(Value::as_str).unwrap_or("max");
    let report_sign = match report_as {
        "max" | "maximise" | "maximize" => 1,
        "min" | "minimise" | "minimize" => -1,
        _ => return Err("choose whether the objective is reported as max or min".into()),
    };
    let default_label = if report_sign < 0 { "C" } else { "Z" };
    let objective_label = v
        .get("objectiveLabel")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .unwrap_or(default_label)
        .to_string();

    let tableau = Tableau {
        headers,
        rows,
        rhs,
        basis,
        obj,
        obj_rhs,
    };
    let pivot_limit = v
        .get("maxPivots")
        .and_then(Value::as_u64)
        .map(|value| value as usize);
    let options = FinishOptions {
        decision_cols,
        artificial_cols,
        objective_label,
        report_sign,
        pivot_limit,
    };
    Ok((tableau, options))
}

fn parse_index_list(value: Option<&Value>, name: &str, width: usize) -> Result<Vec<usize>, String> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let values = value
        .as_array()
        .ok_or_else(|| format!("{name} must be an array of column indices"))?;
    let mut out = Vec::with_capacity(values.len());
    for item in values {
        let index = item
            .as_u64()
            .ok_or_else(|| format!("{name} must contain whole-number column indices"))?
            as usize;
        if index >= width {
            return Err(format!("{name} contains an out-of-range column index"));
        }
        out.push(index);
    }
    Ok(out)
}

fn parse_mval(value: &Value) -> Result<MVal, String> {
    if let Some(text) = value.as_str() {
        if text.trim().is_empty() {
            return Err("enter a number or an expression such as 1 + M".into());
        }
        return parse_mval_text(text);
    }
    if let Some(integer) = value.as_i64() {
        return Ok(MVal::rat(Rat::int(integer)));
    }
    if let Some(float) = value.as_f64() {
        if !float.is_finite() {
            return Err("tableau values must be finite".into());
        }
        return Ok(MVal::rat(Rat::parse(&float.to_string())?));
    }
    Err("tableau values must be numbers or expressions such as 1 + M".into())
}

fn parse_mval_text(input: &str) -> Result<MVal, String> {
    let text = input.trim().replace('−', "-").replace(' ', "");
    if text.is_empty() {
        return Err("enter a number or an expression such as 1 + M".into());
    }
    if !text.contains(['M', 'm']) {
        return Ok(MVal::rat(Rat::parse(input)?));
    }
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0usize;
    let mut constant = Rat::ZERO;
    let mut penalty = Rat::ZERO;
    while index < chars.len() {
        let sign = match chars[index] {
            '+' => {
                index += 1;
                1i64
            }
            '-' => {
                index += 1;
                -1i64
            }
            _ => 1i64,
        };
        let start = index;
        while index < chars.len() && chars[index] != '+' && chars[index] != '-' {
            index += 1;
        }
        if start == index {
            return Err(format!("invalid tableau cell '{input}'"));
        }
        let term: String = chars[start..index].iter().collect();
        let mut coefficient = if term.ends_with('M') || term.ends_with('m') {
            let head = &term[..term.len() - 1];
            if head.is_empty() {
                Rat::ONE
            } else {
                Rat::parse(head)?
            }
        } else {
            Rat::parse(&term)?
        };
        if sign < 0 {
            coefficient = coefficient.neg();
        }
        if term.ends_with('M') || term.ends_with('m') {
            penalty = penalty.add(coefficient)?;
        } else {
            constant = constant.add(coefficient)?;
        }
    }
    Ok(MVal {
        a: constant,
        b: penalty,
    })
}

fn parse_method(v: &Value) -> Result<Method, String> {
    match v.get("method").and_then(Value::as_str).unwrap_or("bigm") {
        "bigm" | "big-m" | "big_m" => Ok(Method::BigM),
        "two-stage" | "two-stage-simplex" | "two_phase" | "two-phase" | "twostage" => {
            Ok(Method::TwoStage)
        }
        other => Err(format!(
            "unknown method '{other}'; choose Big-M or two-stage simplex"
        )),
    }
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
        objective.push(rat_value(c).map_err(|e| format!("objective coefficient {}: {e}", j + 1))?);
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
    fn parses_big_m_cells() {
        let value = super::parse_mval_text("1 - 2M").unwrap();
        assert_eq!(value.a.plain(), "1");
        assert_eq!(value.b.plain(), "-2");
        let value = super::parse_mval_text("-M").unwrap();
        assert!(value.a.is_zero());
        assert_eq!(value.b.plain(), "-1");
        let value = super::parse_mval_text("3/4").unwrap();
        assert_eq!(value.a.plain(), "3/4");
        assert!(value.b.is_zero());
    }

    #[test]
    fn solves_with_two_stage_simplex() {
        let input = r#"{"sense":"min","method":"two-stage","vars":["x","y"],"objective":["2","3"],"constraints":[{"coeffs":["1","1"],"op":">=","rhs":"4"},{"coeffs":["1","3"],"op":">=","rhs":"6"}]}"#;
        let out = super::solve_json(input);
        let value: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value["ok"], true, "{out}");
        assert_eq!(value["method"], "two-stage", "{out}");
        assert_eq!(value["status"], "optimal", "{out}");
        assert_eq!(value["optimum"]["plain"], "9", "{out}");
        assert!(
            value["steps"]
                .as_array()
                .unwrap()
                .iter()
                .any(|step| step["phase"] == 2),
            "{out}"
        );
    }

    #[test]
    fn finishes_a_supplied_tableau() {
        let input = r#"{
            "mode":"tableau",
            "objectiveLabel":"Z",
            "reportAs":"max",
            "headers":["x","y","s1","s2"],
            "basis":[2,3],
            "decisionColumns":[0,1],
            "artificialColumns":[],
            "rows":[["1","1","1","0"],["1","3","0","1"]],
            "rhs":["4","6"],
            "obj":["-3","-2","0","0"],
            "objRhs":"0"
        }"#;
        let out = super::solve_json(input);
        let value: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value["ok"], true, "{out}");
        assert_eq!(value["mode"], "tableau", "{out}");
        assert_eq!(value["status"], "optimal", "{out}");
        assert_eq!(value["optimum"]["plain"], "12", "{out}");
        assert_eq!(value["steps"][0]["kind"], "start", "{out}");
        assert!(value["standardForm"].is_null(), "{out}");
    }

    #[test]
    fn pauses_a_supplied_tableau_after_one_pivot() {
        let input = r#"{
            "mode":"tableau",
            "headers":["x","y","s1","s2"],
            "basis":[2,3],
            "decisionColumns":[0,1],
            "rows":[["1","1","1","0"],["1","3","0","1"]],
            "rhs":["4","6"],
            "obj":["-3","-2","0","0"],
            "objRhs":"0",
            "maxPivots":1
        }"#;
        let out = super::solve_json(input);
        let value: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value["ok"], true, "{out}");
        assert_eq!(value["status"], "paused", "{out}");
        assert_eq!(
            value["steps"].as_array().unwrap().last().unwrap()["kind"],
            "paused",
            "{out}"
        );
    }

    #[test]
    fn rejects_a_bad_m_cell() {
        let input = r#"{
            "mode":"tableau",
            "headers":["x"],
            "basis":[0],
            "rows":[["M2"]],
            "rhs":["1"],
            "obj":["0"],
            "objRhs":"0"
        }"#;
        let out = super::solve_json(input);
        let value: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value["ok"], false, "{out}");
        assert!(
            value["error"].as_str().unwrap().contains("invalid number"),
            "{out}"
        );
    }

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
