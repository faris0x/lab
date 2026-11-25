// Copyright (c) 2026 Faris Alfarhan
// SPDX-License-Identifier: GPL-3.0-only

//! JSON view models and LaTeX generation. The UI renders exact rationals as
//! HTML; LaTeX is generated here from the same structured trace.

use crate::engine::{
    row_op_latex, row_op_plain, var_latex, Method, Ratio, RowOp, Sense, Solution, Source, Standard,
    Status, Step, StepKind, Tableau,
};
use mathcore::{MVal, Rat};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct RatJson {
    pub n: i64,
    pub d: i64,
}

impl From<Rat> for RatJson {
    fn from(r: Rat) -> Self {
        RatJson { n: r.n, d: r.d }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct CellJson {
    pub a: RatJson,
    pub b: RatJson,
}

impl From<MVal> for CellJson {
    fn from(v: MVal) -> Self {
        CellJson {
            a: v.a.into(),
            b: v.b.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct PivotJson {
    pub row: usize,
    pub col: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RatioJson {
    pub row: usize,
    pub value: RatJson,
    pub plain: String,
    pub latex: String,
}

impl From<&Ratio> for RatioJson {
    fn from(r: &Ratio) -> Self {
        RatioJson {
            row: r.row,
            value: r.value.into(),
            plain: r.value.plain(),
            latex: r.value.latex(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RowOpJson {
    pub target: usize,
    pub source: usize,
    pub factor: RatJson,
    pub plain: String,
    pub latex: String,
}

impl From<&RowOp> for RowOpJson {
    fn from(op: &RowOp) -> Self {
        RowOpJson {
            target: op.target,
            source: op.source,
            factor: op.factor.into(),
            plain: row_op_plain(op),
            latex: row_op_latex(op),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepJson {
    pub kind: String,
    pub phase: u8,
    pub index: usize,
    pub headers: Vec<String>,
    pub headers_latex: Vec<String>,
    pub rows: Vec<Vec<CellJson>>,
    pub rhs: Vec<CellJson>,
    pub obj: Vec<CellJson>,
    pub obj_rhs: CellJson,
    pub basis: Vec<String>,
    pub pivot: Option<PivotJson>,
    pub entering: Option<usize>,
    pub leaving: Option<usize>,
    pub ratios: Vec<RatioJson>,
    pub row_ops: Vec<RowOpJson>,
    pub prose: Vec<String>,
    pub prose_latex: Vec<String>,
    pub latex: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolutionJson {
    pub var: String,
    pub value: RatJson,
    pub plain: String,
    pub latex: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ValueJson {
    pub plain: String,
    pub latex: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardFormJson {
    pub objective: String,
    pub objective_latex: String,
    pub constraints: Vec<String>,
    pub constraints_latex: Vec<String>,
    pub objective_row: String,
    pub objective_row_latex: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct LatexJson {
    pub document: String,
    pub blocks: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub ok: bool,
    pub mode: String,
    pub method: String,
    pub status: String,
    pub sense: String,
    pub transformed: bool,
    pub vars: Vec<String>,
    pub objective_label: String,
    pub objective: String,
    pub objective_latex: String,
    pub standard_form: Option<StandardFormJson>,
    pub steps: Vec<StepJson>,
    pub solution: Vec<SolutionJson>,
    pub optimum: Option<ValueJson>,
    pub summary: Vec<String>,
    pub summary_latex: Vec<String>,
    pub latex: LatexJson,
    pub iterations: usize,
}

fn kind_str(k: StepKind) -> &'static str {
    match k {
        StepKind::Start => "start",
        StepKind::Iteration => "iteration",
        StepKind::Transition => "transition",
        StepKind::Paused => "paused",
        StepKind::Optimal => "optimal",
        StepKind::Unbounded => "unbounded",
        StepKind::Infeasible => "infeasible",
    }
}

fn status_str(s: Status) -> &'static str {
    match s {
        Status::Optimal => "optimal",
        Status::Unbounded => "unbounded",
        Status::Infeasible => "infeasible",
        Status::Paused => "paused",
    }
}

fn cell_latex(v: MVal, pivot: bool) -> String {
    let s = v.latex();
    if pivot {
        format!("\\boxed{{{s}}}")
    } else {
        s
    }
}

pub fn tableau_latex(
    tab: &Tableau,
    pivot: Option<(usize, usize)>,
    objective_label: &str,
) -> String {
    let width = tab.headers.len();
    let mut spec = String::from("c|");
    for _ in 0..width {
        spec.push('c');
    }
    spec.push_str("|c");

    let mut out = String::new();
    out.push_str(&format!("\\begin{{array}}{{{spec}}}\n"));
    out.push_str("\\text{b.v.}");
    for h in &tab.headers {
        out.push_str(" & ");
        out.push_str(&h.latex());
    }
    out.push_str(" & \\text{RHS} \\\\ \\hline\n");

    for i in 0..tab.rows.len() {
        let b = tab.basis[i];
        out.push_str(&var_latex(&tab.headers[b].name));
        for (j, cell) in tab.rows[i].iter().enumerate().take(width) {
            out.push_str(" & ");
            out.push_str(&cell_latex(*cell, pivot == Some((i, j))));
        }
        out.push_str(" & ");
        out.push_str(&tab.rhs[i].latex());
        out.push_str(" \\\\\n");
    }

    out.push_str("\\hline\n");
    out.push_str(objective_label);
    for cell in tab.obj.iter().take(width) {
        out.push_str(" & ");
        out.push_str(&cell.latex());
    }
    out.push_str(" & ");
    out.push_str(&tab.obj_rhs.latex());
    out.push_str(" \\\\\n\\end{array}");
    out
}

fn standard_block(standard: &Standard) -> String {
    let mut out = String::from("\\begin{aligned}\n");
    out.push_str("& ");
    out.push_str(&standard.objective_latex);
    out.push_str(" \\\\\n");
    for line in &standard.constraint_latex {
        out.push_str("& ");
        out.push_str(line);
        out.push_str(" \\\\\n");
    }
    out.push_str("& ");
    out.push_str(&standard.objective_row_latex);
    out.push_str(" \\\\\n\\end{aligned}");
    out
}

fn summary_block(sol: &Solution) -> String {
    let (label, names) = match &sol.source {
        Source::Model { model, .. } => (
            if model.sense == Sense::Min { "C" } else { "Z" },
            model.vars.clone(),
        ),
        Source::Tableau {
            start,
            decision_cols,
            objective_label,
            ..
        } => (
            objective_label.as_str(),
            decision_cols
                .iter()
                .map(|&column| start.headers[column].name.clone())
                .collect(),
        ),
    };
    match sol.status {
        Status::Optimal => {
            let value = sol.optimum.unwrap_or(Rat::ZERO);
            let mut bits = Vec::new();
            for (i, v) in sol.values.iter().enumerate() {
                bits.push(format!("{} = {}", var_latex(&names[i]), v.latex()));
            }
            if bits.is_empty() {
                format!("\\text{{Optimal value }} {label} = {}", value.latex())
            } else {
                format!(
                    "\\text{{Optimal value }} {label} = {} \\text{{ at }} {}",
                    value.latex(),
                    bits.join(",\\; ")
                )
            }
        }
        Status::Infeasible => "\\text{The tableau is infeasible.}".to_string(),
        Status::Unbounded => "\\text{The objective is unbounded.}".to_string(),
        Status::Paused => "\\text{Paused; check this tableau before continuing.}".to_string(),
    }
}

fn row_label(source: &Source, phase: u8) -> String {
    if phase == 1 {
        return "W".into();
    }
    match source {
        Source::Model { .. } => "Z".into(),
        Source::Tableau {
            objective_label, ..
        } => objective_label.clone(),
    }
}

fn step_latex(step: &Step, index: usize, source: &Source) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "\\subsection*{{{}}}\n",
        match step.kind {
            StepKind::Start => "Starting tableau".to_string(),
            StepKind::Transition => "Phase 2 · Remove artificials".to_string(),
            StepKind::Paused => "Paused tableau".to_string(),
            StepKind::Iteration if step.phase > 0 => {
                format!("Phase {} · Tableau {}", step.phase, index + 1)
            }
            StepKind::Iteration => format!("Tableau {}", index + 1),
            _ if step.phase > 0 => {
                format!("Phase {} · Final tableau", step.phase)
            }
            _ => "Final tableau".to_string(),
        }
    ));
    for line in &step.note_latex {
        out.push_str(line);
        out.push_str("\n\n");
    }
    out.push_str("\\[\n");
    let objective_label = row_label(source, step.phase);
    out.push_str(&tableau_latex(&step.tableau, step.pivot, &objective_label));
    out.push_str("\n\\]\n");
    out
}

fn method_name(method: Method) -> &'static str {
    match method {
        Method::BigM => "Big-M",
        Method::TwoStage => "Two-stage simplex",
    }
}

fn method_id(method: Method) -> &'static str {
    match method {
        Method::BigM => "bigm",
        Method::TwoStage => "two-stage",
    }
}

fn document(sol: &Solution) -> String {
    let mut out = String::new();
    out.push_str("\\documentclass[11pt]{article}\n");
    out.push_str("\\usepackage[a4paper,margin=2.2cm]{geometry}\n");
    out.push_str("\\usepackage{amsmath,amssymb}\n");
    out.push_str("\\setlength{\\parindent}{0pt}\n");
    out.push_str("\\begin{document}\n");
    out.push_str("\\section*{Linear programming: simplex method}\n");
    match &sol.source {
        Source::Model {
            standard, method, ..
        } => {
            out.push_str("\\textbf{Problem.} $");
            out.push_str(&standard.objective_latex);
            out.push_str("$\\\\[4pt]\n");
            out.push_str("\\textbf{Standard form.}\n");
            out.push_str("\\[\n");
            out.push_str(&standard_block(standard));
            out.push_str("\n\\]\n");
            out.push_str(&format!("\\textbf{{Method.}} {}.\n", method_name(*method)));
        }
        Source::Tableau { .. } => {
            out.push_str("\\textbf{Starting tableau.} The supplied tableau is continued below.\n");
        }
    }

    for (i, step) in sol.steps.iter().enumerate() {
        out.push_str(&step_latex(step, i, &sol.source));
    }

    out.push_str("\\section*{Conclusion}\n");
    for line in &sol.summary_latex {
        out.push_str(line);
        out.push_str("\n\n");
    }
    out.push_str("\\end{document}\n");
    out
}

pub fn render(sol: &Solution) -> Output {
    let mut steps = Vec::with_capacity(sol.steps.len());
    for (i, step) in sol.steps.iter().enumerate() {
        let tab = &step.tableau;
        steps.push(StepJson {
            kind: kind_str(step.kind).to_string(),
            phase: step.phase,
            index: i,
            headers: tab.headers.iter().map(|h| h.name.clone()).collect(),
            headers_latex: tab.headers.iter().map(|h| h.latex()).collect(),
            rows: tab
                .rows
                .iter()
                .map(|row| row.iter().map(|v| CellJson::from(*v)).collect())
                .collect(),
            rhs: tab.rhs.iter().map(|v| CellJson::from(*v)).collect(),
            obj: tab.obj.iter().map(|v| CellJson::from(*v)).collect(),
            obj_rhs: CellJson::from(tab.obj_rhs),
            basis: tab
                .basis
                .iter()
                .map(|&b| tab.headers[b].name.clone())
                .collect(),
            pivot: step.pivot.map(|(row, col)| PivotJson { row, col }),
            entering: step.entering,
            leaving: step.leaving,
            ratios: step.ratios.iter().map(RatioJson::from).collect(),
            row_ops: step.row_ops.iter().map(RowOpJson::from).collect(),
            prose: step.note_plain.clone(),
            prose_latex: step.note_latex.clone(),
            latex: tableau_latex(tab, step.pivot, &row_label(&sol.source, step.phase)),
        });
    }

    let (mode, method, vars, sense, transformed, objective_label, objective, objective_latex) =
        match &sol.source {
            Source::Model {
                model,
                standard,
                method,
            } => (
                "model",
                method_id(*method),
                model.vars.clone(),
                if model.sense == Sense::Max {
                    "max".to_string()
                } else {
                    "min".to_string()
                },
                standard.transformed,
                if model.sense == Sense::Min {
                    "C".to_string()
                } else {
                    "Z".to_string()
                },
                standard.objective_plain.clone(),
                standard.objective_latex.clone(),
            ),
            Source::Tableau {
                start,
                decision_cols,
                objective_label,
                report_sign,
                ..
            } => (
                "tableau",
                "tableau",
                decision_cols
                    .iter()
                    .map(|&column| start.headers[column].name.clone())
                    .collect(),
                if *report_sign < 0 {
                    "min".to_string()
                } else {
                    "max".to_string()
                },
                *report_sign < 0,
                objective_label.clone(),
                "Finish the supplied tableau".to_string(),
                "\\text{Finish the supplied tableau}".to_string(),
            ),
        };

    let solution: Vec<SolutionJson> = vars
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let v = sol.values[i];
            SolutionJson {
                var: name.clone(),
                value: v.into(),
                plain: v.plain(),
                latex: v.latex(),
            }
        })
        .collect();

    let mut blocks = Vec::new();
    if let Source::Model { standard, .. } = &sol.source {
        blocks.push(standard_block(standard));
    }
    for step in &sol.steps {
        blocks.push(tableau_latex(
            &step.tableau,
            step.pivot,
            &row_label(&sol.source, step.phase),
        ));
    }
    blocks.push(summary_block(sol));

    let standard_form = match &sol.source {
        Source::Model { standard, .. } => Some(StandardFormJson {
            objective: standard.objective_plain.clone(),
            objective_latex: standard.objective_latex.clone(),
            constraints: standard.constraint_plain.clone(),
            constraints_latex: standard.constraint_latex.clone(),
            objective_row: standard.objective_row_plain.clone(),
            objective_row_latex: standard.objective_row_latex.clone(),
        }),
        Source::Tableau { .. } => None,
    };

    Output {
        ok: true,
        mode: mode.to_string(),
        method: method.to_string(),
        status: status_str(sol.status).to_string(),
        sense,
        transformed,
        vars,
        objective_label,
        objective,
        objective_latex,
        standard_form,
        steps,
        solution,
        optimum: sol.optimum.map(|v| ValueJson {
            plain: v.plain(),
            latex: v.latex(),
        }),
        summary: sol.summary_plain.clone(),
        summary_latex: sol.summary_latex.clone(),
        latex: LatexJson {
            document: document(sol),
            blocks,
        },
        iterations: sol.iterations,
    }
}
