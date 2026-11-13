//! JSON view models and LaTeX generation. The UI renders exact rationals as
//! HTML; LaTeX is generated here from the same structured trace.

use crate::engine::{
    row_op_latex, row_op_plain, var_latex, Ratio, RowOp, Sense, Solution, Status, Step, StepKind,
    Tableau,
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
    pub status: String,
    pub sense: String,
    pub transformed: bool,
    pub vars: Vec<String>,
    pub objective_label: String,
    pub objective: String,
    pub objective_latex: String,
    pub standard_form: StandardFormJson,
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
        StepKind::Iteration => "iteration",
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

pub fn tableau_latex(tab: &Tableau, pivot: Option<(usize, usize)>) -> String {
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
    out.push('Z');
    for cell in tab.obj.iter().take(width) {
        out.push_str(" & ");
        out.push_str(&cell.latex());
    }
    out.push_str(" & ");
    out.push_str(&tab.obj_rhs.latex());
    out.push_str(" \\\\\n\\end{array}");
    out
}

fn standard_block(sol: &Solution) -> String {
    let mut out = String::from("\\begin{aligned}\n");
    out.push_str("& ");
    out.push_str(&sol.standard.objective_latex);
    out.push_str(" \\\\\n");
    for line in &sol.standard.constraint_latex {
        out.push_str("& ");
        out.push_str(line);
        out.push_str(" \\\\\n");
    }
    out.push_str("& ");
    out.push_str(&sol.standard.objective_row_latex);
    out.push_str(" \\\\\n\\end{aligned}");
    out
}

fn summary_block(sol: &Solution) -> String {
    match sol.status {
        Status::Optimal => {
            let value = sol.optimum.unwrap_or(Rat::ZERO);
            let label = if sol.model.sense == Sense::Min {
                "C"
            } else {
                "Z"
            };
            let mut bits = Vec::new();
            for (i, v) in sol.values.iter().enumerate() {
                bits.push(format!("{} = {}", var_latex(&sol.model.vars[i]), v.latex()));
            }
            format!(
                "\\text{{Optimal value }} {label} = {} \\text{{ at }} {}",
                value.latex(),
                bits.join(",\\; ")
            )
        }
        Status::Infeasible => "\\text{The problem is infeasible.}".to_string(),
        Status::Unbounded => "\\text{The objective is unbounded.}".to_string(),
    }
}

fn step_latex(step: &Step, index: usize) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "\\subsection*{{{}}}\n",
        if step.kind == StepKind::Iteration {
            format!("Tableau {}", index + 1)
        } else {
            "Final tableau".to_string()
        }
    ));
    for line in &step.note_latex {
        out.push_str(line);
        out.push_str("\n\n");
    }
    out.push_str("\\[\n");
    out.push_str(&tableau_latex(&step.tableau, step.pivot));
    out.push_str("\n\\]\n");
    out
}

fn document(sol: &Solution) -> String {
    let mut out = String::new();
    out.push_str("\\documentclass[11pt]{article}\n");
    out.push_str("\\usepackage[a4paper,margin=2.2cm]{geometry}\n");
    out.push_str("\\usepackage{amsmath,amssymb}\n");
    out.push_str("\\setlength{\\parindent}{0pt}\n");
    out.push_str("\\begin{document}\n");
    out.push_str("\\section*{Linear programming: simplex method}\n");
    out.push_str("\\textbf{Problem.} $");
    out.push_str(&sol.standard.objective_latex);
    out.push_str("$\\\\[4pt]\n");
    out.push_str("\\textbf{Standard form.}\n");
    out.push_str("\\[\n");
    out.push_str(&standard_block(sol));
    out.push_str("\n\\]\n");

    for (i, step) in sol.steps.iter().enumerate() {
        out.push_str(&step_latex(step, i));
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
            latex: tableau_latex(tab, step.pivot),
        });
    }

    let solution: Vec<SolutionJson> = sol
        .model
        .vars
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

    let mut blocks = vec![standard_block(sol)];
    for step in &sol.steps {
        blocks.push(tableau_latex(&step.tableau, step.pivot));
    }
    blocks.push(summary_block(sol));

    Output {
        ok: true,
        status: status_str(sol.status).to_string(),
        sense: if sol.model.sense == Sense::Max {
            "max".to_string()
        } else {
            "min".to_string()
        },
        transformed: sol.standard.transformed,
        vars: sol.model.vars.clone(),
        objective_label: if sol.model.sense == Sense::Min {
            "C".to_string()
        } else {
            "Z".to_string()
        },
        objective: sol.standard.objective_plain.clone(),
        objective_latex: sol.standard.objective_latex.clone(),
        standard_form: StandardFormJson {
            objective: sol.standard.objective_plain.clone(),
            objective_latex: sol.standard.objective_latex.clone(),
            constraints: sol.standard.constraint_plain.clone(),
            constraints_latex: sol.standard.constraint_latex.clone(),
            objective_row: sol.standard.objective_row_plain.clone(),
            objective_row_latex: sol.standard.objective_row_latex.clone(),
        },
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
