// Copyright (c) 2026 Faris Alfarhan
// SPDX-License-Identifier: GPL-3.0-only

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
    let mut essential_bits: BTreeSet<String> = BTreeSet::new();

    for minterm in minterms {
        let covering: Vec<&PrimeImplicant> = primes
            .iter()
            .filter(|prime| prime.covers.contains(minterm))
            .collect();
        if covering.len() == 1 && !essential_bits.contains(&covering[0].bits) {
            let prime = covering[0].clone();
            essential_bits.insert(prime.bits.clone());
            essential.push(prime);
        }
    }

    let mut selected = essential.clone();
    selected.extend(exact_cover(&primes, &essential_bits, minterms));
    selected.sort_by(|left, right| left.bits.cmp(&right.bits));

    (primes, essential, selected)
}

fn literal_count(bits: &str) -> usize {
    bits.chars().filter(|bit| *bit != '-').count()
}

fn cover_is_better(candidate: &[usize], best: &[usize], primes: &[PrimeImplicant]) -> bool {
    if candidate.len() != best.len() {
        return candidate.len() < best.len();
    }
    let candidate_literals: usize = candidate
        .iter()
        .map(|index| literal_count(&primes[*index].bits))
        .sum();
    let best_literals: usize = best
        .iter()
        .map(|index| literal_count(&primes[*index].bits))
        .sum();
    if candidate_literals != best_literals {
        return candidate_literals < best_literals;
    }
    let mut candidate_bits: Vec<&str> = candidate
        .iter()
        .map(|index| primes[*index].bits.as_str())
        .collect();
    let mut best_bits: Vec<&str> = best
        .iter()
        .map(|index| primes[*index].bits.as_str())
        .collect();
    candidate_bits.sort_unstable();
    best_bits.sort_unstable();
    candidate_bits < best_bits
}

fn search_cover(
    primes: &[PrimeImplicant],
    candidates: &[usize],
    uncovered: &BTreeSet<usize>,
    chosen: &mut Vec<usize>,
    best: &mut Option<Vec<usize>>,
) {
    if uncovered.is_empty() {
        if best
            .as_ref()
            .is_none_or(|current| cover_is_better(chosen, current, primes))
        {
            *best = Some(chosen.clone());
        }
        return;
    }
    if best
        .as_ref()
        .is_some_and(|current| chosen.len() >= current.len())
    {
        return;
    }

    let pivot = uncovered
        .iter()
        .min_by_key(|minterm| {
            candidates
                .iter()
                .filter(|index| primes[**index].covers.contains(minterm))
                .count()
        })
        .copied();
    let Some(pivot) = pivot else {
        return;
    };

    for &index in candidates {
        if !primes[index].covers.contains(&pivot) {
            continue;
        }
        let mut remaining = uncovered.clone();
        for covered in &primes[index].covers {
            remaining.remove(covered);
        }
        chosen.push(index);
        search_cover(primes, candidates, &remaining, chosen, best);
        chosen.pop();
    }
}

fn exact_cover(
    primes: &[PrimeImplicant],
    essential_bits: &BTreeSet<String>,
    minterms: &[usize],
) -> Vec<PrimeImplicant> {
    let mut uncovered: BTreeSet<usize> = minterms.iter().copied().collect();
    for prime in primes {
        if essential_bits.contains(&prime.bits) {
            for covered in &prime.covers {
                uncovered.remove(covered);
            }
        }
    }
    if uncovered.is_empty() {
        return Vec::new();
    }

    let candidates: Vec<usize> = primes
        .iter()
        .enumerate()
        .filter(|(_, prime)| !essential_bits.contains(&prime.bits))
        .filter(|(_, prime)| prime.covers.iter().any(|value| uncovered.contains(value)))
        .map(|(index, _)| index)
        .collect();

    let mut best = None;
    let mut chosen = Vec::new();
    search_cover(primes, &candidates, &uncovered, &mut chosen, &mut best);
    let mut chosen = best.unwrap_or_default();
    chosen.sort_by(|left, right| primes[*left].bits.cmp(&primes[*right].bits));
    chosen
        .into_iter()
        .map(|index| primes[index].clone())
        .collect()
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

    #[test]
    fn exact_cover_beats_greedy() {
        let result = simplify_boolean(
            "~a&~b&~c&d | ~a&~b&c&~d | ~a&~b&c&d | \
             ~a&b&~c&~d | ~a&b&~c&d | ~a&b&c&~d",
        )
        .unwrap();

        assert_eq!(result.selected.len(), 3);
        let terms: BTreeSet<&str> = result
            .selected
            .iter()
            .map(|prime| prime.expression.as_str())
            .collect();
        assert!(terms.contains("¬a ∧ ¬c ∧ d"));
        assert!(terms.contains("¬a ∧ ¬b ∧ c"));
        assert!(terms.contains("¬a ∧ b ∧ ¬d"));
    }

    #[test]
    fn exact_cover_is_deterministic() {
        let source = "a&b | a&~b | ~a&c";
        let first = simplify_boolean(source).unwrap().minimal;
        let second = simplify_boolean(source).unwrap().minimal;
        assert_eq!(first, second);
    }
}
