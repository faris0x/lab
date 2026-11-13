//! Boolean simplification with prime implicants and canonical forms.

use logic::{collect_variables, evaluate, label, parse, Node};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, Serialize)]
pub struct BooleanRow {
    pub values: Vec<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PrimeImplicant {
    pub bits: String,
    pub expression: String,
    pub covers: Vec<usize>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BooleanResult {
    pub formula: String,
    pub variables: Vec<String>,
    pub rows: Vec<BooleanRow>,
    pub minterms: Vec<usize>,
    pub maxterms: Vec<usize>,
    pub canonical_sop: String,
    pub canonical_pos: String,
    pub primes: Vec<PrimeImplicant>,
    pub essential: Vec<PrimeImplicant>,
    pub selected: Vec<PrimeImplicant>,
    pub minimal: String,
    pub latex: String,
}

#[derive(Deserialize)]
struct Input {
    formula: String,
}

fn binary(value: usize, width: usize) -> String {
    format!("{value:0width$b}", width = width)
}

fn term_expression(bits: &str, variables: &[String]) -> String {
    let mut parts = Vec::new();
    for (index, bit) in bits.chars().enumerate() {
        match bit {
            '-' => {}
            '1' => parts.push(variables[index].clone()),
            _ => parts.push(format!("¬{}", variables[index])),
        }
    }
    if parts.is_empty() {
        "1".into()
    } else {
        parts.join(" ∧ ")
    }
}

fn combine(a: &str, b: &str) -> Option<String> {
    let mut difference = None;
    for (index, (left, right)) in a.chars().zip(b.chars()).enumerate() {
        if left == right {
            continue;
        }
        if difference.is_some() {
            return None;
        }
        difference = Some(index);
    }
    let index = difference?;
    let mut result: Vec<char> = a.chars().collect();
    result[index] = '-';
    Some(result.into_iter().collect())
}

fn covers_term(bits: &str, minterm: usize, width: usize) -> bool {
    let value = binary(minterm, width);
    for (bit, actual) in bits.chars().zip(value.chars()) {
        if bit != '-' && bit != actual {
            return false;
        }
    }
    true
}

fn minimise(
    minterms: &[usize],
    width: usize,
    variables: &[String],
) -> (
    Vec<PrimeImplicant>,
    Vec<PrimeImplicant>,
    Vec<PrimeImplicant>,
) {
    if minterms.is_empty() {
        return (Vec::new(), Vec::new(), Vec::new());
    }
    if minterms.len() == 1usize << width {
        let prime = PrimeImplicant {
            bits: "-".repeat(width),
            expression: "1".into(),
            covers: minterms.to_vec(),
        };
        return (vec![prime.clone()], vec![prime.clone()], vec![prime]);
    }

    let mut current: Vec<String> = Vec::new();
    for value in minterms {
        let term = binary(*value, width);
        if !current.contains(&term) {
            current.push(term);
        }
    }
    let mut prime_bits: BTreeSet<String> = BTreeSet::new();

    while !current.is_empty() {
        let mut used: Vec<String> = Vec::new();
        let mut next: Vec<String> = Vec::new();
        for i in 0..current.len() {
            for j in (i + 1)..current.len() {
                if let Some(merged) = combine(&current[i], &current[j]) {
                    if !used.contains(&current[i]) {
                        used.push(current[i].clone());
                    }
                    if !used.contains(&current[j]) {
                        used.push(current[j].clone());
                    }
                    if !next.contains(&merged) {
                        next.push(merged);
                    }
                }
            }
        }
        for term in &current {
            if !used.contains(term) {
                prime_bits.insert(term.clone());
            }
        }
        current = next;
    }

    let primes: Vec<PrimeImplicant> = prime_bits
        .into_iter()
        .map(|bits| PrimeImplicant {
            expression: term_expression(&bits, variables),
            covers: minterms
                .iter()
                .copied()
                .filter(|value| covers_term(&bits, *value, width))
                .collect(),
            bits,
        })
        .collect();

    let mut essential: Vec<PrimeImplicant> = Vec::new();
    let mut selected: Vec<PrimeImplicant> = Vec::new();
    let mut uncovered: BTreeSet<usize> = minterms.iter().copied().collect();

    for minterm in minterms {
        let covering: Vec<&PrimeImplicant> = primes
            .iter()
            .filter(|prime| prime.covers.contains(minterm))
            .collect();
        if covering.len() == 1 && !selected.iter().any(|prime| prime.bits == covering[0].bits) {
            let prime = covering[0].clone();
            selected.push(prime.clone());
            essential.push(prime.clone());
            for value in &prime.covers {
                uncovered.remove(value);
            }
        }
    }

    while !uncovered.is_empty() {
        let mut best: Option<&PrimeImplicant> = None;
        let mut best_count = 0usize;
        for prime in &primes {
            let count = prime
                .covers
                .iter()
                .filter(|value| uncovered.contains(value))
                .count();
            if count > best_count {
                best = Some(prime);
                best_count = count;
            }
        }
        let Some(best) = best else { break };
        selected.push(best.clone());
        for value in &best.covers {
            uncovered.remove(value);
        }
    }

    (primes, essential, selected)
}

fn latex_expression(expression: &str) -> String {
    expression
        .replace('¬', "\\neg ")
        .replace('∧', "\\land ")
        .replace('∨', "\\lor ")
        .replace('↔', "\\leftrightarrow ")
        .replace('→', "\\to ")
        .replace('⊕', "\\oplus ")
}

pub fn simplify_boolean(source: &str) -> Result<BooleanResult, String> {
    let tree: Node = parse(source)?;
    let mut variables_set = BTreeSet::new();
    collect_variables(&tree, &mut variables_set);
    let variables: Vec<String> = variables_set.into_iter().collect();
    if variables.len() > 6 {
        return Err("Use at most six variables in the simplifier.".into());
    }

    let mut rows = Vec::new();
    let mut minterms = Vec::new();
    let mut maxterms = Vec::new();
    let total = 1usize << variables.len();
    for mask in 0..total {
        let mut values = HashMap::new();
        for (bit, name) in variables.iter().enumerate() {
            let value = mask & (1 << (variables.len() - 1 - bit)) != 0;
            values.insert(name.clone(), value);
        }
        let output = evaluate(&tree, &values);
        let mut row: Vec<bool> = variables.iter().map(|name| values[name]).collect();
        row.push(output);
        rows.push(BooleanRow { values: row });
        if output {
            minterms.push(mask);
        } else {
            maxterms.push(mask);
        }
    }

    let canonical_sop = if minterms.is_empty() {
        "0".into()
    } else {
        minterms
            .iter()
            .map(|value| {
                binary(*value, variables.len())
                    .chars()
                    .enumerate()
                    .map(|(index, bit)| {
                        if bit == '1' {
                            variables[index].clone()
                        } else {
                            format!("¬{}", variables[index])
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ∧ ")
            })
            .collect::<Vec<_>>()
            .join(" ∨ ")
    };

    let canonical_pos = if maxterms.is_empty() {
        "1".into()
    } else {
        maxterms
            .iter()
            .map(|value| {
                let clause = binary(*value, variables.len())
                    .chars()
                    .enumerate()
                    .map(|(index, bit)| {
                        if bit == '1' {
                            format!("¬{}", variables[index])
                        } else {
                            variables[index].clone()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ∨ ");
                if maxterms.len() == 1 {
                    clause
                } else {
                    format!("({clause})")
                }
            })
            .collect::<Vec<_>>()
            .join(" ∧ ")
    };

    let (primes, essential, selected) = minimise(&minterms, variables.len(), &variables);
    let minimal = if selected.is_empty() {
        "0".into()
    } else {
        selected
            .iter()
            .map(|prime| prime.expression.clone())
            .collect::<Vec<_>>()
            .join(" ∨ ")
    };

    let latex = build_latex(
        &label(&tree),
        &variables,
        &minterms,
        &maxterms,
        &canonical_sop,
        &canonical_pos,
        &primes,
        &essential,
        &selected,
        &minimal,
        &rows,
    );

    Ok(BooleanResult {
        formula: source.trim().to_string(),
        variables,
        rows,
        minterms,
        maxterms,
        canonical_sop,
        canonical_pos,
        primes,
        essential,
        selected,
        minimal,
        latex,
    })
}

#[allow(clippy::too_many_arguments)]
fn build_latex(
    source: &str,
    variables: &[String],
    minterms: &[usize],
    maxterms: &[usize],
    canonical_sop: &str,
    canonical_pos: &str,
    primes: &[PrimeImplicant],
    essential: &[PrimeImplicant],
    _selected: &[PrimeImplicant],
    minimal: &str,
    rows: &[BooleanRow],
) -> String {
    let header = variables
        .iter()
        .cloned()
        .chain(std::iter::once("F".to_string()))
        .collect::<Vec<_>>()
        .join(" & ");
    let body = rows
        .iter()
        .map(|row| {
            row.values
                .iter()
                .map(|value| if *value { "1" } else { "0" })
                .collect::<Vec<_>>()
                .join(" & ")
        })
        .collect::<Vec<_>>()
        .join(" \\\\\n");
    let prime_rows = primes
        .iter()
        .map(|prime| {
            format!(
                "{} & {}",
                latex_expression(&prime.expression),
                prime
                    .covers
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join(" \\\\\n");
    let essential_list = if essential.is_empty() {
        "none".to_string()
    } else {
        essential
            .iter()
            .map(|prime| format!("${}$", latex_expression(&prime.expression)))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let minterm_list = if minterms.is_empty() {
        "-".to_string()
    } else {
        minterms
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };
    let maxterm_list = if maxterms.is_empty() {
        "-".to_string()
    } else {
        maxterms
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };
    let spec = format!("{}c", "c|".repeat(variables.len()));

    format!(
        "\\documentclass[11pt]{{article}}\n\
\\usepackage[a4paper,margin=2.2cm]{{geometry}}\n\
\\usepackage{{amsmath,amssymb}}\n\
\\setlength{{\\parindent}}{{0pt}}\n\
\\begin{{document}}\n\
\\section*{{Boolean simplification}}\n\
\\[\n{}\n\\]\n\
\\subsection*{{Truth table}}\n\
\\[\n\\begin{{array}}{{{spec}}}\n{header} \\\\ \\hline\n{body}\n\\end{{array}}\n\\]\n\
Minterms: $\\Sigma m({minterm_list})$. Maxterms: $\\Pi M({maxterm_list})$.\n\n\
\\subsection*{{Canonical forms}}\n\
SOP: \\[\n{}\n\\]\n\
POS: \\[\n{}\n\\]\n\n\
\\subsection*{{Prime implicants}}\n\
\\[\n\\begin{{array}}{{l|l}}\n\\text{{Term}} & \\text{{Minterms}} \\\\ \\hline\n{}\n\\end{{array}}\n\\]\n\
Essential prime implicants: {}.\n\n\
\\subsection*{{Minimal sum of products}}\n\
\\[\n{}\n\\]\n\
\\end{{document}}\n",
        latex_expression(source),
        latex_expression(canonical_sop),
        latex_expression(canonical_pos),
        if prime_rows.is_empty() {
            "\\text{none}".to_string()
        } else {
            prime_rows
        },
        essential_list,
        latex_expression(minimal)
    )
}

pub fn solve_json(input: &str) -> String {
    let parsed: Input = match serde_json::from_str(input) {
        Ok(value) => value,
        Err(error) => {
            return serde_json::json!({ "ok": false, "error": format!("Invalid request: {error}") })
                .to_string()
        }
    };
    match simplify_boolean(&parsed.formula) {
        Ok(output) => serde_json::json!({ "ok": true, "result": output }).to_string(),
        Err(error) => serde_json::json!({ "ok": false, "error": error }).to_string(),
    }
}

#[wasm_bindgen]
pub fn solve(input: &str) -> String {
    solve_json(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simplifies_redundant_expression() {
        let result = simplify_boolean("a & b | a & ~b").unwrap();
        assert_eq!(result.variables, vec!["a", "b"]);
        assert_eq!(result.minimal, "a");
    }

    #[test]
    fn handles_constants() {
        assert_eq!(simplify_boolean("a").unwrap().minimal, "a");
        assert_eq!(simplify_boolean("a | ~a").unwrap().minimal, "1");
        assert_eq!(simplify_boolean("a & ~a").unwrap().minimal, "0");
    }

    #[test]
    fn lists_minterms_in_order() {
        let result = simplify_boolean("a & b").unwrap();
        assert_eq!(result.minterms, vec![3]);
        assert_eq!(result.maxterms, vec![0, 1, 2]);
    }

    #[test]
    fn serialises_camel_case_field_names() {
        let output = solve_json(r#"{"formula":"a & b"}"#);
        assert!(output.contains("\"canonicalSop\""));
        assert!(output.contains("\"canonicalPos\""));
    }

    #[test]
    fn rejects_too_many_variables() {
        assert!(simplify_boolean("a|b|c|d|e|g|h")
            .unwrap_err()
            .contains("at most six"));
    }
}
