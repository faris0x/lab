use crate::engine::{matrix_latex, vector_latex, Matrix, Step};
use mathcore::Rat;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepJson {
    pub kind: String,
    pub matrix_latex: String,
    pub description: String,
    pub description_latex: String,
    pub pivot: Option<[usize; 2]>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatrixResultJson {
    pub label: String,
    pub label_latex: String,
    pub latex: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScalarResultJson {
    pub label: String,
    pub label_latex: String,
    pub plain: String,
    pub latex: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VectorResultJson {
    pub label: String,
    pub label_latex: String,
    pub latex: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultJson {
    pub matrix: Option<MatrixResultJson>,
    pub scalar: Option<ScalarResultJson>,
    pub vectors: Vec<VectorResultJson>,
    pub text: Vec<String>,
    pub text_latex: Vec<String>,
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
    pub input_latex: String,
    pub steps: Vec<StepJson>,
    pub result: ResultJson,
    pub summary: Vec<String>,
    pub summary_latex: Vec<String>,
    pub latex: LatexJson,
}

pub struct LabeledMatrix {
    pub label: String,
    pub label_latex: String,
    pub matrix: Matrix,
}

pub struct LabeledScalar {
    pub label: String,
    pub label_latex: String,
    pub value: Rat,
}

pub struct LabeledVector {
    pub label: String,
    pub label_latex: String,
    pub values: Vec<Rat>,
}

pub struct ResultData {
    pub matrix: Option<LabeledMatrix>,
    pub scalar: Option<LabeledScalar>,
    pub vectors: Vec<LabeledVector>,
    pub text: Vec<String>,
    pub text_latex: Vec<String>,
}

pub struct Outcome {
    pub operation: &'static str,
    pub input: Matrix,
    pub steps: Vec<Step>,
    pub result: ResultData,
    pub summary: Vec<String>,
    pub summary_latex: Vec<String>,
}

fn operation_title(operation: &str) -> &'static str {
    match operation {
        "rref" => "reduced row echelon form",
        "det" => "determinant",
        "inverse" => "inverse",
        "nullspace" => "nullspace",
        "solve" => "linear system",
        _ => "matrix operation",
    }
}

fn step_json(step: &Step) -> StepJson {
    StepJson {
        kind: step.kind.to_string(),
        matrix_latex: matrix_latex(&step.matrix, step.pivot),
        description: step.description.clone(),
        description_latex: step.description_latex.clone(),
        pivot: step.pivot.map(|(r, c)| [r, c]),
    }
}

fn document(outcome: &Outcome, result: &ResultJson) -> String {
    let mut out = String::new();
    out.push_str("\\documentclass[11pt]{article}\n");
    out.push_str("\\usepackage[a4paper,margin=2.2cm]{geometry}\n");
    out.push_str("\\usepackage{amsmath,amssymb}\n");
    out.push_str("\\setlength{\\parindent}{0pt}\n");
    out.push_str("\\begin{document}\n");
    out.push_str(&format!(
        "\\section*{{Matrix toolkit: {}}}\n",
        operation_title(outcome.operation)
    ));

    out.push_str("\\textbf{Input.}\n");
    out.push_str("\\[\nA = ");
    out.push_str(&matrix_latex(&outcome.input, None));
    out.push_str("\n\\]\n");

    for (i, step) in outcome.steps.iter().enumerate() {
        out.push_str(&format!("\\subsection*{{Step {}}}\n", i + 1));
        out.push_str(&step.description_latex);
        out.push_str("\n\\[\n");
        out.push_str(&matrix_latex(&step.matrix, step.pivot));
        out.push_str("\n\\]\n");
    }

    out.push_str("\\section*{Result}\n");
    if let Some(matrix) = &result.matrix {
        out.push_str(&format!("\\[\n{} = ", matrix.label_latex));
        out.push_str(&matrix.latex);
        out.push_str("\n\\]\n");
    }
    if let Some(scalar) = &result.scalar {
        out.push_str(&format!(
            "\\[\n{} = {}\n\\]\n",
            scalar.label_latex, scalar.latex
        ));
    }
    for vector in &result.vectors {
        out.push_str(&format!("\\[\n{} = ", vector.label_latex));
        out.push_str(&vector.latex);
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
    let steps: Vec<StepJson> = outcome.steps.iter().map(step_json).collect();

    let result = ResultJson {
        matrix: outcome.result.matrix.as_ref().map(|m| MatrixResultJson {
            label: m.label.clone(),
            label_latex: m.label_latex.clone(),
            latex: matrix_latex(&m.matrix, None),
        }),
        scalar: outcome.result.scalar.as_ref().map(|s| ScalarResultJson {
            label: s.label.clone(),
            label_latex: s.label_latex.clone(),
            plain: s.value.plain(),
            latex: s.value.latex(),
        }),
        vectors: outcome
            .result
            .vectors
            .iter()
            .map(|v| VectorResultJson {
                label: v.label.clone(),
                label_latex: v.label_latex.clone(),
                latex: vector_latex(&v.values),
            })
            .collect(),
        text: outcome.result.text.clone(),
        text_latex: outcome.result.text_latex.clone(),
    };

    let mut blocks = vec![format!(
        "A = {}",
        matrix_latex(&outcome.input, None)
    )];
    for step in &outcome.steps {
        blocks.push(matrix_latex(&step.matrix, step.pivot));
    }
    if let Some(matrix) = &result.matrix {
        blocks.push(format!("{} = {}", matrix.label_latex, matrix.latex));
    }
    if let Some(scalar) = &result.scalar {
        blocks.push(format!(
            "{} = {}",
            scalar.label_latex, scalar.latex
        ));
    }
    for vector in &result.vectors {
        blocks.push(format!("{} = {}", vector.label_latex, vector.latex));
    }

    let document_text = document(outcome, &result);

    Output {
        ok: true,
        operation: outcome.operation.to_string(),
        input_latex: matrix_latex(&outcome.input, None),
        steps,
        result,
        summary: outcome.summary.clone(),
        summary_latex: outcome.summary_latex.clone(),
        latex: LatexJson {
            document: document_text,
            blocks,
        },
    }
}
