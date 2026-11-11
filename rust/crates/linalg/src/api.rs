use crate::engine::{
    determinant, inverse, nullspace, rref, solve, Matrix, SolveKind,
};
use crate::render::{
    self, LabeledMatrix, LabeledScalar, LabeledVector, Outcome, ResultData,
};
use mathcore::Rat;
use serde_json::Value;

enum Operation {
    Rref,
    Det,
    Inverse,
    Nullspace,
    Solve,
}

impl Operation {
    fn name(&self) -> &'static str {
        match self {
            Operation::Rref => "rref",
            Operation::Det => "det",
            Operation::Inverse => "inverse",
            Operation::Nullspace => "nullspace",
            Operation::Solve => "solve",
        }
    }
}

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
    let operation = match v.get("operation").and_then(Value::as_str) {
        Some("rref") => Operation::Rref,
        Some("det") => Operation::Det,
        Some("inverse") => Operation::Inverse,
        Some("nullspace") => Operation::Nullspace,
        Some("solve") => Operation::Solve,
        _ => return Err("choose an operation".into()),
    };
    let matrix = parse_matrix(v)?;
    let rhs = if matches!(operation, Operation::Solve) {
        parse_rhs(v, matrix.rows)?
    } else {
        Vec::new()
    };

    let outcome = match operation {
        Operation::Rref => {
            let reduced = rref(
                matrix.clone(),
                "Start with A. Work left to right: choose a nonzero pivot in each column, make it 1, and clear the rest of that column.",
                "Start with $A$. Work left to right: choose a nonzero pivot in each column, make it $1$, and clear the rest of that column.",
            )?;
            let rank = reduced.pivots.len();
            let nullity = matrix.cols - rank;
            let pivot_list = human_list(&reduced.pivots);
            let free = free_columns(matrix.cols, &reduced.pivots);
            let mut summary = vec![format!(
                "The rank is {rank}; the nullity is {nullity}."
            )];
            summary.push(format!(
                "Pivot columns: {}. Free columns: {}.",
                if pivot_list.is_empty() { "none".to_string() } else { pivot_list },
                human_list_or_none(&free)
            ));
            let mut summary_latex = vec![format!(
                "$\\operatorname{{rank}}(A) = {rank}$, $\\operatorname{{nullity}}(A) = {nullity}$"
            )];
            summary_latex.push(format!(
                "$\\text{{Pivot columns: }}{}$; $\\text{{free columns: }}{}$",
                if reduced.pivots.is_empty() {
                    "\\text{none}".to_string()
                } else {
                    human_list_latex(&reduced.pivots)
                },
                if free.is_empty() {
                    "\\text{none}".to_string()
                } else {
                    human_list_latex(&free)
                }
            ));
            Outcome {
                operation: operation.name(),
                input: matrix,
                steps: reduced.steps,
                result: ResultData {
                    matrix: Some(LabeledMatrix {
                        label: "RREF(A)".into(),
                        label_latex: "\\mathrm{RREF}(A)".into(),
                        matrix: reduced.matrix,
                    }),
                    scalar: None,
                    vectors: Vec::new(),
                    text: Vec::new(),
                    text_latex: Vec::new(),
                },
                summary,
                summary_latex,
            }
        }
        Operation::Det => {
            let det = determinant(matrix.clone())?;
            let value = det.value;
            let mut summary = vec![format!("det(A) = {}.", value.plain())];
            let mut summary_latex = vec![format!("$\\det(A) = {}$", value.latex())];
            if value.is_zero() {
                summary.push("A is singular.".into());
                summary_latex.push("$A$ is singular.".into());
            }
            Outcome {
                operation: operation.name(),
                input: matrix,
                steps: det.steps,
                result: ResultData {
                    matrix: None,
                    scalar: Some(LabeledScalar {
                        label: "det(A)".into(),
                        label_latex: "\\det(A)".into(),
                        value,
                    }),
                    vectors: Vec::new(),
                    text: Vec::new(),
                    text_latex: Vec::new(),
                },
                summary,
                summary_latex,
            }
        }
        Operation::Inverse => {
            let (inverse_matrix, reduced) = inverse(matrix.clone())?;
            let summary;
            let summary_latex;
            let result = if let Some(inv) = inverse_matrix {
                summary = vec![
                    "Row-reducing [A | I] turned the left block into I; the right block is A^{-1}.".into(),
                ];
                summary_latex = vec![
                    "Row-reducing $[A \\mid I]$ turned the left block into $I$; the right block is $A^{-1}$.".into(),
                ];
                ResultData {
                    matrix: Some(LabeledMatrix {
                        label: "A⁻¹".into(),
                        label_latex: "A^{-1}".into(),
                        matrix: inv,
                    }),
                    scalar: None,
                    vectors: Vec::new(),
                    text: Vec::new(),
                    text_latex: Vec::new(),
                }
            } else {
                summary = vec![
                    "The left block could not become I, so A is singular and has no inverse.".into(),
                ];
                summary_latex = vec![
                    "The left block could not become $I$, so $A$ is singular and has no inverse.".into(),
                ];
                ResultData {
                    matrix: None,
                    scalar: None,
                    vectors: Vec::new(),
                    text: vec!["No inverse exists.".into()],
                    text_latex: vec!["No inverse exists.".into()],
                }
            };
            Outcome {
                operation: operation.name(),
                input: matrix,
                steps: reduced.steps,
                result,
                summary,
                summary_latex,
            }
        }
        Operation::Nullspace => {
            let space = nullspace(matrix.clone())?;
            let dimension = space.basis.len();
            let free = free_columns(matrix.cols, &space.rref.pivots);
            let mut summary = vec![format!("The nullspace has dimension {dimension}.")];
            let mut summary_latex = vec![format!(
                "$\\dim\\operatorname{{null}}(A) = {dimension}$"
            )];
            let mut vectors = Vec::new();
            for (i, values) in space.basis.iter().enumerate() {
                vectors.push(LabeledVector {
                    label: format!("v{}", i + 1),
                    label_latex: format!("\\mathbf{{v}}_{{{}}}", i + 1),
                    values: values.clone(),
                });
            }
            if dimension == 0 {
                summary.push(
                    "Every column is a pivot column, so the nullspace is the zero subspace."
                        .into(),
                );
                summary_latex.push(
                    "Every column is a pivot column, so the nullspace is the zero subspace."
                        .into(),
                );
            } else {
                summary.push(format!(
                    "Free columns: {}; each one gives one basis vector.",
                    human_list(&free)
                ));
                summary_latex.push(format!(
                    "$\\text{{Free columns: }}{}$; each one gives one basis vector.",
                    human_list_latex(&free)
                ));
            }
            Outcome {
                operation: operation.name(),
                input: matrix,
                steps: space.rref.steps,
                result: ResultData {
                    matrix: None,
                    scalar: None,
                    vectors,
                    text: Vec::new(),
                    text_latex: Vec::new(),
                },
                summary,
                summary_latex,
            }
        }
        Operation::Solve => {
            let solved = solve(matrix.clone(), &rhs)?;
            let cols = matrix.cols;
            let free = free_columns(cols, &solved.rref.pivots);
            let mut offending = None;
            for (i, row) in solved.rref.matrix.data.iter().enumerate() {
                if row[..cols].iter().all(|v| v.is_zero()) && !row[cols].is_zero() {
                    offending = Some((i, row[cols]));
                }
            }

            let summary;
            let summary_latex;
            let mut vectors = Vec::new();
            let mut text = Vec::new();
            let mut text_latex = Vec::new();

            match solved.kind {
                SolveKind::Unique(values) => {
                    summary = vec![
                        "Every column is a pivot column, so the solution is unique.".into(),
                        "The values of the variables are shown above.".into(),
                    ];
                    summary_latex = vec![
                        "Every column is a pivot column, so the solution is unique.".into(),
                        "The values of the variables are shown above.".into(),
                    ];
                    vectors.push(LabeledVector {
                        label: "x".into(),
                        label_latex: "\\mathbf{x}".into(),
                        values,
                    });
                }
                SolveKind::Infinite {
                    particular,
                    nullspace,
                } => {
                    let parameters = free.len();
                    summary = vec![
                        format!(
                            "Free columns: {}; the solution has {parameters} parameter{}.",
                            human_list(&free),
                            if parameters == 1 { "" } else { "s" }
                        ),
                        "A particular solution and a nullspace basis are shown above.".into(),
                    ];
                    summary_latex = vec![
                        format!(
                            "$\\text{{Free columns: }}{}$; the solution has {parameters} parameter{}.",
                            human_list_latex(&free),
                            if parameters == 1 { "" } else { "s" }
                        ),
                        "A particular solution and a nullspace basis are shown above.".into(),
                    ];
                    vectors.push(LabeledVector {
                        label: "xₚ".into(),
                        label_latex: "\\mathbf{x}_p".into(),
                        values: particular,
                    });
                    for (i, values) in nullspace.iter().enumerate() {
                        vectors.push(LabeledVector {
                            label: format!("v{}", i + 1),
                            label_latex: format!("\\mathbf{{v}}_{{{}}}", i + 1),
                            values: values.clone(),
                        });
                    }
                    text.push(
                        "Choose a parameter for each free column and add the corresponding nullspace vector to xₚ."
                            .into(),
                    );
                    text_latex.push(
                        "Choose a parameter for each free column and add the corresponding nullspace vector to $\\mathbf{x}_p$."
                            .into(),
                    );
                }
                SolveKind::Inconsistent => {
                    if let Some((row, value)) = offending {
                        summary = vec![format!(
                            "Row {} reduces to 0 = {}, so the system is inconsistent.",
                            row + 1,
                            value.plain()
                        )];
                        summary_latex = vec![format!(
                            "Row {} reduces to $0 = {}$, so the system is inconsistent.",
                            row + 1,
                            value.latex()
                        )];
                        text.push(format!(
                            "The equation 0 = {} cannot be satisfied.",
                            value.plain()
                        ));
                        text_latex.push(format!(
                            "The equation $0 = {}$ cannot be satisfied.",
                            value.latex()
                        ));
                    } else {
                        summary = vec!["The system is inconsistent.".into()];
                        summary_latex = vec!["The system is inconsistent.".into()];
                        text.push(
                            "At least one row has no coefficients but a nonzero right-hand side."
                                .into(),
                        );
                        text_latex.push(
                            "At least one row has no coefficients but a nonzero right-hand side."
                                .into(),
                        );
                    }
                }
            }

            Outcome {
                operation: operation.name(),
                input: matrix,
                steps: solved.rref.steps,
                result: ResultData {
                    matrix: None,
                    scalar: None,
                    vectors,
                    text,
                    text_latex,
                },
                summary,
                summary_latex,
            }
        }
    };

    Ok(render::render(&outcome))
}

fn parse_matrix(v: &Value) -> Result<Matrix, String> {
    let rows_value = v
        .get("matrix")
        .and_then(Value::as_array)
        .ok_or("provide a matrix")?;
    if rows_value.is_empty() || rows_value.len() > 6 {
        return Err("use between 1 and 6 rows".into());
    }
    let cols = rows_value[0]
        .as_array()
        .ok_or("each matrix row must be an array")?
        .len();
    if cols == 0 || cols > 6 {
        return Err("use between 1 and 6 columns".into());
    }
    let mut data = Vec::with_capacity(rows_value.len());
    for (i, row_value) in rows_value.iter().enumerate() {
        let row = row_value
            .as_array()
            .ok_or_else(|| format!("row {} must be an array", i + 1))?;
        if row.len() != cols {
            return Err(format!("row {} has the wrong number of entries", i + 1));
        }
        let mut parsed = Vec::with_capacity(cols);
        for (j, entry) in row.iter().enumerate() {
            parsed.push(
                rat_value(entry).map_err(|e| {
                    format!("entry ({}, {}): {e}", i + 1, j + 1)
                })?,
            );
        }
        data.push(parsed);
    }
    Matrix::new(rows_value.len(), cols, data)
}

fn parse_rhs(v: &Value, rows: usize) -> Result<Vec<Rat>, String> {
    let values = v
        .get("rhs")
        .and_then(Value::as_array)
        .ok_or("provide a right-hand side")?;
    if values.len() != rows {
        return Err("the right-hand side needs one entry per row".into());
    }
    let mut out = Vec::with_capacity(rows);
    for (i, value) in values.iter().enumerate() {
        out.push(rat_value(value).map_err(|e| format!("right-hand side {}: {e}", i + 1))?);
    }
    Ok(out)
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
            return Err("entries must be finite".into());
        }
        return Rat::parse(&f.to_string());
    }
    Err("entries must be numbers".into())
}

fn free_columns(cols: usize, pivots: &[usize]) -> Vec<usize> {
    (0..cols).filter(|c| !pivots.contains(c)).collect()
}

fn human_list_or_none(values: &[usize]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        human_list(values)
    }
}

fn human_list(values: &[usize]) -> String {
    values
        .iter()
        .map(|v| (v + 1).to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn human_list_latex(values: &[usize]) -> String {
    values
        .iter()
        .map(|v| (v + 1).to_string())
        .collect::<Vec<_>>()
        .join(",\\; ")
}
