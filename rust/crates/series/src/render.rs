use crate::detect::{detect, GeneralTerm};
use crate::expr::{
    constant_value, derivative, div, expr_latex, expr_plain, factorial, node_count, rat_expr,
    rat_latex, rat_plain, series_latex, series_plain, substitute, term_latex, Expr,
};
use crate::parser::parse_expression;
use serde::Serialize;

pub const MAX_ORDER: usize = 12;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesStep {
    pub k: usize,
    pub derivative_latex: String,
    pub value_latex: String,
    pub coefficient_latex: String,
    pub coefficient_plain: String,
    pub term_latex: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralTermOutput {
    pub term_latex: String,
    pub term_plain: String,
    pub sum_latex: String,
    pub note: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesResult {
    pub source: String,
    pub function_latex: String,
    pub center: String,
    pub center_latex: String,
    pub order: usize,
    pub kind: String,
    pub series_latex: String,
    pub steps: Vec<SeriesStep>,
    pub summary: Vec<String>,
    pub summary_latex: Vec<String>,
    pub latex: String,
    pub general: Option<GeneralTermOutput>,
}

fn general_output(term: GeneralTerm) -> GeneralTermOutput {
    GeneralTermOutput {
        term_latex: term.term_latex,
        term_plain: term.term_plain,
        sum_latex: term.sum_latex,
        note: term.note,
    }
}

pub fn expand_series(
    source: &str,
    center_text: &str,
    order: usize,
) -> Result<SeriesResult, String> {
    if order > MAX_ORDER {
        return Err(format!(
            "Choose a whole number of terms between 0 and {MAX_ORDER}."
        ));
    }
    let tree = parse_expression(source)?;
    let center_expression = parse_expression(center_text)?;
    let center = constant_value(&center_expression)?.ok_or_else(|| {
        "The centre must be a rational number, for example 0, 1 or 1/2.".to_string()
    })?;

    let kind = if center.is_zero() {
        "maclaurin"
    } else {
        "taylor"
    };
    let label = if kind == "maclaurin" {
        "Maclaurin"
    } else {
        "Taylor"
    };
    let center_plain = rat_plain(center);
    let center_latex = rat_latex(center);

    let mut steps = Vec::new();
    let mut terms: Vec<(Expr, usize)> = Vec::new();
    let mut current = tree.clone();
    for k in 0..=order {
        let value = substitute(&current, center)?;
        let coefficient = div(value.clone(), rat_expr(factorial(k)))?;
        steps.push(SeriesStep {
            k,
            derivative_latex: expr_latex(&current),
            value_latex: expr_latex(&value),
            coefficient_latex: expr_latex(&coefficient),
            coefficient_plain: expr_plain(&coefficient),
            term_latex: term_latex(&coefficient, center, k),
        });
        terms.push((coefficient, k));
        if k < order {
            current = derivative(&current)?;
            if node_count(&current) > 100_000 {
                return Err(
                    "This expansion grows too large to display. Try a lower degree.".into(),
                );
            }
        }
    }

    let general = detect(&tree, center, label);
    let series = series_latex(&terms, center);
    let series_plain_text = series_plain(&terms, center);
    let rows = steps
        .iter()
        .map(|step| {
            format!(
                "{} & {} & {} & {}",
                step.k, step.derivative_latex, step.value_latex, step.coefficient_latex
            )
        })
        .collect::<Vec<_>>()
        .join(" \\\\\n");

    let general_section = match &general {
        Some(term) => format!(
            "\\subsection*{{General term}}\n\\[\nT_n = {}\n\\]\n\\[\n{}\n\\]\n{}\n",
            term.term_latex, term.sum_latex, term.note
        ),
        None => "\\subsection*{General term}\nNo simple closed form was detected for this series. The nth coefficient is still $c_n = f^{(n)}(a)/n!$.\n".to_string(),
    };

    let latex = format!(
        "\\documentclass[11pt]{{article}}\n\
\\usepackage[a4paper,margin=2.2cm]{{geometry}}\n\
\\usepackage{{amsmath,amssymb}}\n\
\\setlength{{\\parindent}}{{0pt}}\n\
\\begin{{document}}\n\
\\section*{{{label} series}}\n\
\\subsection*{{Function and centre}}\n\
\\[\nf(x) = {}\n\\]\n\
The {label} expansion about $x = {center_latex}$ to degree {order}.\n\
\\subsection*{{Derivatives and coefficients}}\n\
\\[\n\\begin{{array}}{{c|l|l|l}}\nk & f^{{(k)}}(x) & f^{{(k)}}(a) & c_k \\\\\n\\hline\n{rows}\n\\end{{array}}\n\\]\n\
\\subsection*{{Series}}\n\
\\[\nT_{{{order}}}(x) = {series}\n\\]\n\
{general_section}\
\\end{{document}}\n",
        expr_latex(&tree)
    );

    let mut summary = vec![
        format!(
            "{label} series of f(x) = {}, about x = {center_plain}, degree {order}.",
            expr_plain(&tree)
        ),
        format!("Coefficients use c_k = f^(k)(a)/k!, with a = {center_plain}."),
        format!("The expanded polynomial is {series_plain_text}."),
    ];
    let mut summary_latex = vec![
        format!("f(x) = {}", expr_latex(&tree)),
        format!("a = {center_latex}"),
        format!("T_{{{order}}}(x) = {series}"),
    ];
    if let Some(term) = &general {
        summary.push(format!("The general term is {}.", term.term_plain));
        summary_latex.push(format!("T_n = {}", term.term_latex));
    }

    Ok(SeriesResult {
        source: source.trim().to_string(),
        function_latex: expr_latex(&tree),
        center: center_plain,
        center_latex,
        order,
        kind: kind.to_string(),
        series_latex: series,
        steps,
        summary,
        summary_latex,
        latex,
        general: general.map(general_output),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coefficients(source: &str, center: &str, order: usize) -> Vec<String> {
        expand_series(source, center, order)
            .unwrap()
            .steps
            .into_iter()
            .map(|step| step.coefficient_plain)
            .collect()
    }

    #[test]
    fn expands_exponential() {
        let result = expand_series("exp(x)", "0", 5).unwrap();
        assert_eq!(result.kind, "maclaurin");
        assert_eq!(
            coefficients("exp(x)", "0", 5),
            vec!["1", "1", "1/2", "1/6", "1/24", "1/120"]
        );
        assert!(result.series_latex.contains("x^{5}"));
    }

    #[test]
    fn reports_the_general_term_of_sine() {
        let result = expand_series("sin(x)", "0", 7).unwrap();
        let general = result.general.expect("sine has a general term");
        assert_eq!(general.term_latex, "\\frac{(-1)^{n} x^{2n+1}}{(2n+1)!}");
        assert_eq!(
            general.sum_latex,
            "\\sum_{n=0}^{\\infty} \\frac{(-1)^{n} x^{2n+1}}{(2n+1)!}"
        );
    }

    #[test]
    fn reports_no_general_term_for_non_rational_centres() {
        let result = expand_series("sin(x)", "1", 3).unwrap();
        assert!(result.general.is_none());
    }

    #[test]
    fn expands_sine() {
        assert_eq!(
            coefficients("sin(x)", "0", 7),
            vec!["0", "1", "0", "-1/6", "0", "1/120", "0", "-1/5040"]
        );
    }

    #[test]
    fn expands_cosine() {
        assert_eq!(
            coefficients("cos(x)", "0", 4),
            vec!["1", "0", "-1/2", "0", "1/24"]
        );
    }

    #[test]
    fn expands_logarithm() {
        assert_eq!(
            coefficients("ln(1+x)", "0", 4),
            vec!["0", "1", "-1/2", "1/3", "-1/4"]
        );
    }

    #[test]
    fn expands_geometric() {
        assert_eq!(
            coefficients("1/(1-x)", "0", 4),
            vec!["1", "1", "1", "1", "1"]
        );
    }

    #[test]
    fn expands_implicit_logarithm() {
        assert_eq!(
            coefficients("ln(1-3x)", "0", 3),
            vec!["0", "-3", "-9/2", "-9"]
        );
    }

    #[test]
    fn handles_exponential_notation() {
        assert_eq!(coefficients("e^x", "0", 3), vec!["1", "1", "1/2", "1/6"]);
    }

    #[test]
    fn expands_polynomial_about_one() {
        assert_eq!(coefficients("x^2", "1", 3), vec!["1", "2", "1", "0"]);
    }

    #[test]
    fn expands_square_root_about_one() {
        assert_eq!(
            coefficients("sqrt(x)", "1", 3),
            vec!["1", "1/2", "-1/8", "1/16"]
        );
    }

    #[test]
    fn keeps_transcendental_values_symbolic() {
        assert_eq!(
            coefficients("sin(x)", "1", 2),
            vec!["sin(1)", "cos(1)", "-1/2*sin(1)"]
        );
    }

    #[test]
    fn expands_fractional_power() {
        assert_eq!(
            coefficients("(1+x)^(1/2)", "0", 3),
            vec!["1", "1/2", "-1/8", "1/16"]
        );
    }

    #[test]
    fn rejects_invalid_logarithm() {
        assert!(expand_series("ln(x)", "0", 3)
            .unwrap_err()
            .contains("positive"));
    }

    #[test]
    fn rejects_negative_square_root() {
        assert!(expand_series("sqrt(x)", "-1", 3)
            .unwrap_err()
            .contains("negative"));
    }

    #[test]
    fn rejects_large_order() {
        assert!(expand_series("sin(x)", "0", 13)
            .unwrap_err()
            .contains("between 0 and 12"));
    }

    #[test]
    fn rejects_non_rational_center() {
        assert!(expand_series("sin(x)", "pi", 3)
            .unwrap_err()
            .contains("rational"));
    }
}
