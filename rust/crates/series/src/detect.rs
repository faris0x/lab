//! Heuristic detection of a closed-form general term from exact coefficients.

use crate::expr::{
    constant_value, derivative, factorial, node_count, rat_latex, rat_plain, shift_latex,
    shift_plain, substitute, Expr, FunctionName,
};
use mathcore::Rat;

pub const DETECT_ORDER: usize = 8;
const MAX_DETECT_NODES: usize = 4_000;

#[derive(Clone, Debug)]
pub struct GeneralTerm {
    pub term_latex: String,
    pub term_plain: String,
    pub sum_latex: String,
    pub note: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Denominator {
    Factorial,
    Linear,
    One,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SignPattern {
    Constant,
    Alternating,
    AlternatingShifted,
}

fn sign_at(pattern: SignPattern, m: usize) -> i64 {
    match pattern {
        SignPattern::Constant => 1,
        SignPattern::Alternating => {
            if m.is_multiple_of(2) {
                1
            } else {
                -1
            }
        }
        SignPattern::AlternatingShifted => {
            if m.is_multiple_of(2) {
                -1
            } else {
                1
            }
        }
    }
}

fn denominator_at(denominator: Denominator, k: usize) -> Option<Rat> {
    match denominator {
        Denominator::Factorial => Some(factorial(k)),
        Denominator::Linear => {
            if k == 0 {
                None
            } else {
                Some(Rat::int(k as i64))
            }
        }
        Denominator::One => Some(Rat::ONE),
    }
}

fn index_latex(step: usize, start: usize) -> String {
    match (step, start) {
        (1, 0) => "n".into(),
        (1, 1) => "n+1".into(),
        (1, other) => format!("n+{other}"),
        (2, 0) => "2n".into(),
        (2, 1) => "2n+1".into(),
        (2, other) => format!("2n+{other}"),
        (_, 0) => format!("{step}n"),
        (_, other) => format!("{step}n+{other}"),
    }
}

fn index_plain(step: usize, start: usize) -> String {
    match (step, start) {
        (1, 0) => "n".into(),
        (1, 1) => "n+1".into(),
        (1, other) => format!("n+{other}"),
        (2, 0) => "2n".into(),
        (2, 1) => "2n+1".into(),
        (2, other) => format!("2n+{other}"),
        (_, 0) => format!("{step}n"),
        (_, other) => format!("{step}n+{other}"),
    }
}

fn denominator_latex(denominator: Denominator, index: &str) -> Option<String> {
    match denominator {
        Denominator::Factorial => {
            if index == "n" {
                Some("n!".into())
            } else {
                Some(format!("({index})!"))
            }
        }
        Denominator::Linear => Some(format!("({index})")),
        Denominator::One => None,
    }
}

fn coefficient_sequence(tree: &Expr, center: Rat) -> Option<Vec<Rat>> {
    let mut coefficients = Vec::new();
    let mut current = tree.clone();
    for k in 0..=DETECT_ORDER {
        let value = substitute(&current, center).ok()?;
        let derivative_value = constant_value(&value).ok()??;
        let coefficient = derivative_value.div(factorial(k)).ok()?;
        coefficients.push(coefficient);
        if k < DETECT_ORDER {
            current = derivative(&current).ok()?;
            if node_count(&current) > MAX_DETECT_NODES {
                return None;
            }
        }
    }
    Some(coefficients)
}

fn contains_tan(value: &Expr) -> bool {
    match value {
        Expr::Num(_) | Expr::Var | Expr::Const(_) => false,
        Expr::Neg(child) => contains_tan(child),
        Expr::Add(left, right)
        | Expr::Sub(left, right)
        | Expr::Mul(left, right)
        | Expr::Div(left, right) => contains_tan(left) || contains_tan(right),
        Expr::Pow(base, _) => contains_tan(base),
        Expr::Fn(name, arg) => *name == FunctionName::Tan || contains_tan(arg),
    }
}

pub fn detect(tree: &Expr, center: Rat, label: &str) -> Option<GeneralTerm> {
    if contains_tan(tree) {
        return None;
    }
    let coefficients = coefficient_sequence(tree, center)?;
    let nonzero: Vec<usize> = coefficients
        .iter()
        .enumerate()
        .filter(|(_, value)| !value.is_zero())
        .map(|(index, _)| index)
        .collect();

    if nonzero.is_empty() {
        return Some(GeneralTerm {
            term_latex: "0".into(),
            term_plain: "0".into(),
            sum_latex: "0".into(),
            note: "Every coefficient is zero, so the series is identically zero.".into(),
        });
    }
    if nonzero.len() < 4 {
        return None;
    }

    let start = nonzero[0];
    let step = nonzero[1] - nonzero[0];
    if step == 0 || nonzero.last().copied().unwrap_or(0) + step <= DETECT_ORDER {
        return None;
    }
    for (m, k) in nonzero.iter().enumerate() {
        if *k != start + step * m {
            return None;
        }
    }

    let denominators = [
        Denominator::Factorial,
        Denominator::Linear,
        Denominator::One,
    ];
    let signs = [
        SignPattern::Constant,
        SignPattern::Alternating,
        SignPattern::AlternatingShifted,
    ];

    for denominator in denominators {
        for pattern in signs {
            let mut constant: Option<Rat> = None;
            let mut valid = true;
            for (m, k) in nonzero.iter().enumerate() {
                let Some(divisor) = denominator_at(denominator, *k) else {
                    valid = false;
                    break;
                };
                let mut candidate = coefficients[*k].mul(divisor).ok()?;
                if sign_at(pattern, m) < 0 {
                    candidate = candidate.neg();
                }
                match constant {
                    None => constant = Some(candidate),
                    Some(existing) if existing != candidate => {
                        valid = false;
                        break;
                    }
                    _ => {}
                }
            }
            if !valid {
                continue;
            }
            let Some(mut coefficient) = constant else {
                continue;
            };
            let mut sign_pattern = pattern;
            let mut negative = false;
            if coefficient.is_negative() {
                coefficient = coefficient.abs();
                match sign_pattern {
                    SignPattern::Constant => negative = true,
                    SignPattern::Alternating => sign_pattern = SignPattern::AlternatingShifted,
                    SignPattern::AlternatingShifted => sign_pattern = SignPattern::Alternating,
                }
            }

            let index = index_latex(step, start);
            let index_plain = index_plain(step, start);
            let shift = shift_latex(center);
            let shift_plain = shift_plain(center);
            let variable_latex = format!("{shift}^{{{index}}}");
            let variable_plain = format!("{shift_plain}^({index_plain})");

            let mut parts: Vec<String> = Vec::new();
            let mut plain_parts: Vec<String> = Vec::new();
            match sign_pattern {
                SignPattern::Constant => {}
                SignPattern::Alternating => {
                    parts.push("(-1)^{n}".into());
                    plain_parts.push("(-1)^n".into());
                }
                SignPattern::AlternatingShifted => {
                    parts.push("(-1)^{n+1}".into());
                    plain_parts.push("(-1)^(n+1)".into());
                }
            }
            if coefficient != Rat::ONE {
                parts.push(rat_latex(coefficient));
                plain_parts.push(rat_plain(coefficient));
            }
            parts.push(variable_latex);
            plain_parts.push(variable_plain);

            let mut numerator = parts.join(" ");
            let mut plain_numerator = plain_parts.join("*");
            if negative {
                numerator.insert(0, '-');
                plain_numerator.insert(0, '-');
            }

            let denominator_text = denominator_latex(denominator, &index);
            let term_latex = match denominator_text {
                Some(divisor) => format!("\\frac{{{numerator}}}{{{divisor}}}"),
                None => numerator,
            };
            let term_plain = match denominator {
                Denominator::Factorial => {
                    if index_plain == "n" {
                        format!("{plain_numerator}/n!")
                    } else {
                        format!("{plain_numerator}/({index_plain})!")
                    }
                }
                Denominator::Linear => format!("{plain_numerator}/({index_plain})"),
                Denominator::One => plain_numerator,
            };
            let sum_latex = format!("\\sum_{{n=0}}^{{\\infty}} {term_latex}");
            let note = format!(
                "General term detected from the {label} coefficients and checked against every computed term up to degree {DETECT_ORDER}."
            );
            return Some(GeneralTerm {
                term_latex,
                term_plain,
                sum_latex,
                note,
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_expression;

    fn find(source: &str) -> Option<GeneralTerm> {
        let tree = parse_expression(source).unwrap();
        detect(&tree, Rat::ZERO, "Maclaurin")
    }

    #[test]
    fn detects_sine() {
        let term = find("sin(x)").unwrap();
        assert_eq!(term.term_latex, "\\frac{(-1)^{n} x^{2n+1}}{(2n+1)!}");
    }

    #[test]
    fn detects_cosine() {
        let term = find("cos(x)").unwrap();
        assert_eq!(term.term_latex, "\\frac{(-1)^{n} x^{2n}}{(2n)!}");
    }

    #[test]
    fn detects_exponential() {
        let term = find("exp(x)").unwrap();
        assert_eq!(term.term_latex, "\\frac{x^{n}}{n!}");
    }

    #[test]
    fn detects_logarithm() {
        let term = find("ln(1+x)").unwrap();
        assert_eq!(term.term_latex, "\\frac{(-1)^{n} x^{n+1}}{(n+1)}");
    }

    #[test]
    fn detects_geometric() {
        let term = find("1/(1-x)").unwrap();
        assert_eq!(term.term_latex, "x^{n}");
    }

    #[test]
    fn ignores_polynomials() {
        assert!(find("x^2").is_none());
    }
}
