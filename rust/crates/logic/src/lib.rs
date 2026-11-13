//! Propositional logic parsing, truth tables and LaTeX output.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
#[cfg(feature = "export")]
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    Var(String),
    Not(Box<Node>),
    And(Box<Node>, Box<Node>),
    Or(Box<Node>, Box<Node>),
    Xor(Box<Node>, Box<Node>),
    Imp(Box<Node>, Box<Node>),
    Iff(Box<Node>, Box<Node>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Var,
    Not,
    And,
    Or,
    Xor,
    Imp,
    Iff,
    Open,
    Close,
}

#[derive(Clone, Debug)]
struct Token {
    kind: Kind,
    value: String,
}

#[derive(Clone, Debug)]
struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

fn normalize(source: &str) -> String {
    source
        .replace('¬', "~")
        .replace('∧', "&")
        .replace('∨', "|")
        .replace('→', "->")
        .replace('↔', "<->")
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let text = normalize(source);
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index..].starts_with(&['<', '-', '>']) {
            tokens.push(Token {
                kind: Kind::Iff,
                value: "<->".into(),
            });
            index += 3;
        } else if chars[index..].starts_with(&['-', '>']) {
            tokens.push(Token {
                kind: Kind::Imp,
                value: "->".into(),
            });
            index += 2;
        } else {
            let character = chars[index];
            let kind = match character {
                '~' | '!' => Kind::Not,
                '&' => Kind::And,
                '^' => Kind::Xor,
                '|' | 'v' | 'V' => Kind::Or,
                '(' => Kind::Open,
                ')' => Kind::Close,
                '⊤' | '1' => {
                    tokens.push(Token {
                        kind: Kind::Var,
                        value: "t".into(),
                    });
                    index += 1;
                    continue;
                }
                '⊥' | '0' => {
                    tokens.push(Token {
                        kind: Kind::Var,
                        value: "f".into(),
                    });
                    index += 1;
                    continue;
                }
                other if other.is_ascii_alphabetic() => {
                    tokens.push(Token {
                        kind: Kind::Var,
                        value: other.to_ascii_lowercase().to_string(),
                    });
                    index += 1;
                    continue;
                }
                other => {
                    return Err(format!(
                        "Unknown symbol “{other}”. Use letters, ~, &, |, ->, <-> and parentheses."
                    ))
                }
            };
            tokens.push(Token {
                kind,
                value: character.to_string(),
            });
            index += 1;
        }
    }
    if tokens.is_empty() {
        return Err("Enter a formula.".into());
    }
    Ok(tokens)
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    fn take(&mut self, kind: Option<Kind>) -> Result<Token, String> {
        let token = self
            .tokens
            .get(self.index)
            .cloned()
            .ok_or_else(|| "The formula ends early.".to_string())?;
        if let Some(expected) = kind {
            if token.kind != expected {
                return Err(format!(
                    "Expected {} but found “{}”.",
                    kind_name(expected),
                    token.value
                ));
            }
        }
        self.index += 1;
        Ok(token)
    }

    fn parse_iff(&mut self) -> Result<Node, String> {
        let mut left = self.parse_imp()?;
        while self.peek().map(|token| token.kind) == Some(Kind::Iff) {
            self.take(Some(Kind::Iff))?;
            let right = self.parse_imp()?;
            left = Node::Iff(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_imp(&mut self) -> Result<Node, String> {
        let left = self.parse_or()?;
        if self.peek().map(|token| token.kind) == Some(Kind::Imp) {
            self.take(Some(Kind::Imp))?;
            let right = self.parse_imp()?;
            return Ok(Node::Imp(Box::new(left), Box::new(right)));
        }
        Ok(left)
    }

    fn parse_or(&mut self) -> Result<Node, String> {
        let mut left = self.parse_xor()?;
        while self.peek().map(|token| token.kind) == Some(Kind::Or) {
            self.take(Some(Kind::Or))?;
            let right = self.parse_xor()?;
            left = Node::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_xor(&mut self) -> Result<Node, String> {
        let mut left = self.parse_and()?;
        while self.peek().map(|token| token.kind) == Some(Kind::Xor) {
            self.take(Some(Kind::Xor))?;
            let right = self.parse_and()?;
            left = Node::Xor(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Node, String> {
        let mut left = self.parse_not()?;
        while self.peek().map(|token| token.kind) == Some(Kind::And) {
            self.take(Some(Kind::And))?;
            let right = self.parse_not()?;
            left = Node::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Node, String> {
        if self.peek().map(|token| token.kind) == Some(Kind::Not) {
            self.take(Some(Kind::Not))?;
            return Ok(Node::Not(Box::new(self.parse_not()?)));
        }
        self.parse_atom()
    }

    fn parse_atom(&mut self) -> Result<Node, String> {
        let token = self
            .peek()
            .cloned()
            .ok_or_else(|| "The formula ends early.".to_string())?;
        match token.kind {
            Kind::Open => {
                self.take(Some(Kind::Open))?;
                let inner = self.parse_iff()?;
                self.take(Some(Kind::Close))?;
                Ok(inner)
            }
            Kind::Var => {
                self.take(Some(Kind::Var))?;
                Ok(Node::Var(token.value))
            }
            _ => Err(format!("Unexpected “{}”.", token.value)),
        }
    }
}

fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Var => "variable",
        Kind::Not => "not",
        Kind::And => "and",
        Kind::Or => "or",
        Kind::Xor => "xor",
        Kind::Imp => "implication",
        Kind::Iff => "biconditional",
        Kind::Open => "open parenthesis",
        Kind::Close => "close parenthesis",
    }
}

pub fn parse(source: &str) -> Result<Node, String> {
    let tokens = tokenize(source)?;
    let mut parser = Parser { tokens, index: 0 };
    let tree = parser.parse_iff()?;
    if let Some(token) = parser.peek() {
        return Err(format!("Unexpected “{}” after the formula.", token.value));
    }
    Ok(tree)
}

pub fn label(node: &Node) -> String {
    match node {
        Node::Var(name) => name.clone(),
        Node::Not(child) => {
            let inner = label(child);
            if needs_parens(child) {
                format!("¬({inner})")
            } else {
                format!("¬{inner}")
            }
        }
        Node::And(left, right) => format!("{} ∧ {}", wrap(left), wrap(right)),
        Node::Or(left, right) => format!("{} ∨ {}", wrap(left), wrap(right)),
        Node::Xor(left, right) => format!("{} ⊕ {}", wrap(left), wrap(right)),
        Node::Imp(left, right) => format!("{} → {}", wrap(left), wrap(right)),
        Node::Iff(left, right) => format!("{} ↔ {}", wrap(left), wrap(right)),
    }
}

fn needs_parens(node: &Node) -> bool {
    !matches!(node, Node::Var(_) | Node::Not(_))
}

fn wrap(node: &Node) -> String {
    let text = label(node);
    if needs_parens(node) {
        format!("({text})")
    } else {
        text
    }
}

pub fn evaluate(node: &Node, values: &HashMap<String, bool>) -> bool {
    match node {
        Node::Var(name) => match name.as_str() {
            "t" => true,
            "f" => false,
            other => *values.get(other).unwrap_or(&false),
        },
        Node::Not(child) => !evaluate(child, values),
        Node::And(left, right) => evaluate(left, values) && evaluate(right, values),
        Node::Or(left, right) => evaluate(left, values) || evaluate(right, values),
        Node::Xor(left, right) => evaluate(left, values) != evaluate(right, values),
        Node::Imp(left, right) => !evaluate(left, values) || evaluate(right, values),
        Node::Iff(left, right) => evaluate(left, values) == evaluate(right, values),
    }
}

pub fn depth(node: &Node) -> usize {
    match node {
        Node::Var(_) => 0,
        Node::Not(child) => 1 + depth(child),
        Node::And(left, right)
        | Node::Or(left, right)
        | Node::Xor(left, right)
        | Node::Imp(left, right)
        | Node::Iff(left, right) => 1 + depth(left).max(depth(right)),
    }
}

pub fn collect_variables(node: &Node, variables: &mut BTreeSet<String>) {
    match node {
        Node::Var(name) => {
            if name != "t" && name != "f" {
                variables.insert(name.clone());
            }
        }
        Node::Not(child) => collect_variables(child, variables),
        Node::And(left, right)
        | Node::Or(left, right)
        | Node::Xor(left, right)
        | Node::Imp(left, right)
        | Node::Iff(left, right) => {
            collect_variables(left, variables);
            collect_variables(right, variables);
        }
    }
}

fn walk(node: &Node, subexpressions: &mut Vec<Node>, variables: &mut BTreeSet<String>) {
    match node {
        Node::Var(name) => {
            if name != "t" && name != "f" {
                variables.insert(name.clone());
            }
        }
        Node::Not(child) => {
            walk(child, subexpressions, variables);
            subexpressions.push(node.clone());
        }
        Node::And(left, right)
        | Node::Or(left, right)
        | Node::Xor(left, right)
        | Node::Imp(left, right)
        | Node::Iff(left, right) => {
            walk(left, subexpressions, variables);
            walk(right, subexpressions, variables);
            subexpressions.push(node.clone());
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableColumn {
    pub label: String,
    pub key: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct TableRow {
    pub values: Vec<bool>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LogicResult {
    pub formula: String,
    pub variables: Vec<String>,
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
    pub verdict: String,
    pub latex: String,
}

#[derive(Clone)]
struct Unique {
    node: Node,
    label: String,
    key: String,
}

#[derive(Deserialize)]
struct Input {
    formula: String,
}

pub fn build(source: &str) -> Result<LogicResult, String> {
    let tree = parse(source)?;
    let mut variables = BTreeSet::new();
    let mut subexpressions = Vec::new();
    walk(&tree, &mut subexpressions, &mut variables);
    let variable_names: Vec<String> = variables.into_iter().collect();

    let mut unique: Vec<Unique> = Vec::new();
    for node in subexpressions {
        let text = label(&node);
        if !unique.iter().any(|entry| entry.key == text) {
            unique.push(Unique {
                node,
                label: text.clone(),
                key: text,
            });
        }
    }
    unique.sort_by(|a, b| {
        let da = depth(&a.node);
        let db = depth(&b.node);
        if da == db {
            a.key.len().cmp(&b.key.len())
        } else {
            da.cmp(&db)
        }
    });

    let is_plain_variable = matches!(&tree, Node::Var(name) if name != "t" && name != "f");
    if !is_plain_variable {
        let root_key = label(&tree);
        let mut others: Vec<Unique> = unique
            .into_iter()
            .filter(|entry| !(entry.key == root_key && entry.node == tree))
            .collect();
        others.push(Unique {
            node: tree.clone(),
            label: root_key.clone(),
            key: root_key,
        });
        unique = others;
    }

    let mut columns: Vec<TableColumn> = variable_names
        .iter()
        .map(|name| TableColumn {
            label: name.clone(),
            key: name.clone(),
        })
        .collect();
    for entry in &unique {
        columns.push(TableColumn {
            label: entry.label.clone(),
            key: entry.key.clone(),
        });
    }

    let total = 1usize << variable_names.len();
    let mut rows = Vec::with_capacity(total);
    for mask in 0..total {
        let mut values = HashMap::new();
        for (bit, name) in variable_names.iter().enumerate() {
            let value = mask & (1 << (variable_names.len() - 1 - bit)) != 0;
            values.insert(name.clone(), value);
        }
        let mut row_values: Vec<bool> = variable_names.iter().map(|name| values[name]).collect();
        for entry in &unique {
            row_values.push(evaluate(&entry.node, &values));
        }
        rows.push(TableRow { values: row_values });
    }

    let finals: Vec<bool> = rows
        .iter()
        .map(|row| *row.values.last().unwrap_or(&false))
        .collect();
    let verdict = if finals.iter().all(|value| *value) {
        "tautology"
    } else if finals.iter().all(|value| !*value) {
        "contradiction"
    } else {
        "contingent"
    };

    let latex = build_latex(source.trim(), &columns, &rows, verdict);

    Ok(LogicResult {
        formula: source.trim().to_string(),
        variables: variable_names,
        columns,
        rows,
        verdict: verdict.to_string(),
        latex,
    })
}

fn tex_label(value: &str) -> String {
    value
        .replace('¬', "\\neg ")
        .replace('∧', "\\land ")
        .replace('∨', "\\lor ")
        .replace('→', "\\to ")
        .replace('↔', "\\leftrightarrow ")
}

fn build_latex(formula: &str, columns: &[TableColumn], rows: &[TableRow], verdict: &str) -> String {
    let header = columns
        .iter()
        .map(|column| tex_label(&column.label))
        .collect::<Vec<_>>()
        .join(" & ");
    let body = rows
        .iter()
        .map(|row| {
            row.values
                .iter()
                .map(|value| if *value { "\\top" } else { "\\bot" })
                .collect::<Vec<_>>()
                .join(" & ")
        })
        .collect::<Vec<_>>()
        .join(" \\\\\n");
    let verdict_text = match verdict {
        "tautology" => "The formula is a tautology: every row is true.",
        "contradiction" => "The formula is a contradiction: every row is false.",
        _ => "The formula is contingent: it is true in some rows and false in others.",
    };
    let formula_latex = formula
        .replace("<->", "\\leftrightarrow ")
        .replace("->", "\\to ")
        .replace(['&', '∧'], "\\land ")
        .replace(['|', '∨'], "\\lor ")
        .replace('^', "\\oplus ")
        .replace(['~', '¬'], "\\neg ");
    let spec = if columns.is_empty() {
        "c".to_string()
    } else {
        format!("{}c", "c|".repeat(columns.len() - 1))
    };

    format!(
        "\\documentclass[11pt]{{article}}\n\
\\usepackage[a4paper,margin=2.2cm]{{geometry}}\n\
\\usepackage{{amsmath,amssymb}}\n\
\\usepackage{{array}}\n\
\\setlength{{\\parindent}}{{0pt}}\n\
\\begin{{document}}\n\
\\section*{{Truth table}}\n\
\\[\n{formula_latex}\n\\]\n\
\\[\n\\begin{{array}}{{{spec}}}\n{header} \\\\ \\hline\n{body}\n\\end{{array}}\n\\]\n\
{verdict_text}\n\
\\end{{document}}\n"
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
    match build(&parsed.formula) {
        Ok(output) => serde_json::json!({ "ok": true, "result": output }).to_string(),
        Err(error) => serde_json::json!({ "ok": false, "error": error }).to_string(),
    }
}

#[cfg(feature = "export")]
#[wasm_bindgen]
pub fn solve(input: &str) -> String {
    solve_json(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(pairs: &[(&str, bool)]) -> HashMap<String, bool> {
        pairs
            .iter()
            .map(|(name, value)| (name.to_string(), *value))
            .collect()
    }

    #[test]
    fn normalises_unicode_operators() {
        let tree = parse("p ∧ ¬q").unwrap();
        assert!(evaluate(&tree, &values(&[("p", true), ("q", false)])));
    }

    #[test]
    fn rejects_unknown_symbols() {
        assert!(parse("p ? q").unwrap_err().contains("Unknown symbol"));
    }

    #[test]
    fn detects_tautology() {
        let result = build("p | ~p").unwrap();
        assert_eq!(result.variables, vec!["p"]);
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.verdict, "tautology");
    }

    #[test]
    fn detects_contradiction() {
        assert_eq!(build("p & ~p").unwrap().verdict, "contradiction");
    }

    #[test]
    fn classifies_implication() {
        let result = build("p -> q").unwrap();
        assert_eq!(result.variables, vec!["p", "q"]);
        assert_eq!(result.rows.len(), 4);
        assert_eq!(result.verdict, "contingent");
    }

    #[test]
    fn handles_constants() {
        assert_eq!(build("⊤").unwrap().verdict, "tautology");
        assert_eq!(build("⊥").unwrap().verdict, "contradiction");
    }
}
