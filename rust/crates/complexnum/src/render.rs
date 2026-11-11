use crate::engine::{exact_latex, Outcome};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepJson {
    pub description: String,
    pub description_latex: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PointJson {
    pub label: String,
    pub label_latex: String,
    pub x: f64,
    pub y: f64,
    pub kind: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CircleJson {
    pub radius: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ArgandJson {
    pub points: Vec<PointJson>,
    pub circles: Vec<CircleJson>,
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
    pub operation: String,
    pub z_latex: String,
    pub w_latex: Option<String>,
    pub steps: Vec<StepJson>,
    pub summary: Vec<String>,
    pub summary_latex: Vec<String>,
    pub lines: Vec<String>,
    pub latex: LatexJson,
    pub argand: ArgandJson,
}

fn title(operation: &str) -> &'static str {
    match operation {
        "arithmetic" => "complex arithmetic",
        "polar" => "polar form",
        "power" => "de Moivre's theorem",
        "roots" => "roots of a complex number",
        _ => "complex numbers",
    }
}

fn document(outcome: &Outcome) -> String {
    let mut out = String::new();
    out.push_str("\\documentclass[11pt]{article}\n");
    out.push_str("\\usepackage[a4paper,margin=2.2cm]{geometry}\n");
    out.push_str("\\usepackage{amsmath,amssymb}\n");
    out.push_str("\\setlength{\\parindent}{0pt}\n");
    out.push_str("\\begin{document}\n");
    out.push_str(&format!(
        "\\section*{{Complex numbers: {}}}\n",
        title(outcome.operation)
    ));
    out.push_str("\\textbf{Input.}\n");
    out.push_str("\\[\nz = ");
    out.push_str(&exact_latex(outcome.z));
    out.push_str("\n\\]\n");
    if let Some(w) = outcome.w {
        out.push_str("\\[\nw = ");
        out.push_str(&exact_latex(w));
        out.push_str("\n\\]\n");
    }
    for step in &outcome.steps {
        out.push_str(&step.description_latex);
        out.push_str("\n\n");
    }
    out.push_str("\\section*{Result}\n");
    for line in &outcome.lines {
        out.push_str("\\[\n");
        out.push_str(line);
        out.push_str("\n\\]\n");
    }
    for line in &outcome.summary_latex {
        out.push_str(line);
        out.push_str("\n\n");
    }
    out.push_str("\\end{document}\n");
    out
}

pub fn render(outcome: &Outcome) -> Output {
    let argand = ArgandJson {
        points: outcome
            .argand
            .points
            .iter()
            .map(|p| PointJson {
                label: p.label.clone(),
                label_latex: p.label_latex.clone(),
                x: p.x,
                y: p.y,
                kind: p.kind.to_string(),
            })
            .collect(),
        circles: outcome
            .argand
            .circles
            .iter()
            .map(|c| CircleJson { radius: c.radius })
            .collect(),
    };

    Output {
        ok: true,
        operation: outcome.operation.to_string(),
        z_latex: exact_latex(outcome.z),
        w_latex: outcome.w.map(exact_latex),
        steps: outcome
            .steps
            .iter()
            .map(|s| StepJson {
                description: s.description.clone(),
                description_latex: s.description_latex.clone(),
            })
            .collect(),
        summary: outcome.summary.clone(),
        summary_latex: outcome.summary_latex.clone(),
        lines: outcome.lines.clone(),
        latex: LatexJson {
            document: document(outcome),
            blocks: outcome.lines.clone(),
        },
        argand,
    }
}
