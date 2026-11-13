use crate::expr::{
    add, constant_value, div, fn_expr, mul, neg, pow, sub, Const, Expr, FunctionName, RAT_HALF,
};
use mathcore::Rat;

fn normalize_source(source: &str) -> String {
    let mut text = source.replace("\\left", "").replace("\\right", "");
    for (pattern, replacement) in [
        ("\\,", " "),
        ("\\;", " "),
        ("\\:", " "),
        ("\\!", " "),
        ("\\quad", " "),
        ("\\qquad", " "),
        ("\\displaystyle", " "),
        ("\\textstyle", " "),
    ] {
        text = text.replace(pattern, replacement);
    }
    for (pattern, replacement) in [
        ("\\sinh", "sinh"),
        ("\\cosh", "cosh"),
        ("\\sin", "sin"),
        ("\\cos", "cos"),
        ("\\tan", "tan"),
        ("\\exp", "exp"),
        ("\\ln", "ln"),
        ("\\sqrt", "sqrt "),
    ] {
        text = text.replace(pattern, replacement);
    }
    for (pattern, replacement) in [
        ("\\cdot", "*"),
        ("\\times", "*"),
        ("\\ast", "*"),
        ("\\div", "/"),
        ("\\pi", "pi"),
    ] {
        text = text.replace(pattern, replacement);
    }
    text = text
        .replace(['×', '·'], "*")
        .replace('÷', "/")
        .replace(['−', '–'], "-")
        .replace('π', "pi")
        .replace('√', "sqrt ")
        .replace(['{', '}', '\\'], "");
    text
}

struct Parser {
    source: Vec<char>,
    index: usize,
}

impl Parser {
    fn skip(&mut self) {
        while self
            .source
            .get(self.index)
            .map(|character| character.is_whitespace())
            .unwrap_or(false)
        {
            self.index += 1;
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.skip();
        self.source.get(self.index).copied()
    }

    fn take(&mut self) -> Option<char> {
        self.skip();
        let value = self.source.get(self.index).copied();
        if value.is_some() {
            self.index += 1;
        }
        value
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        if self.take() != Some(expected) {
            return Err(format!("Expected “{expected}”."));
        }
        Ok(())
    }

    fn parse_sum(&mut self) -> Result<Expr, String> {
        let mut value = self.parse_term()?;
        loop {
            match self.peek() {
                Some('+') => {
                    self.take();
                    value = add(value, self.parse_term()?)?;
                }
                Some('-') => {
                    self.take();
                    value = sub(value, self.parse_term()?)?;
                }
                _ => return Ok(value),
            }
        }
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut value = self.parse_unary()?;
        loop {
            match self.peek() {
                Some('*') => {
                    self.take();
                    value = mul(value, self.parse_unary()?)?;
                }
                Some('/') => {
                    self.take();
                    value = div(value, self.parse_unary()?)?;
                }
                Some(next) if starts_primary(next) => {
                    value = mul(value, self.parse_unary()?)?;
                }
                _ => return Ok(value),
            }
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some('-') => {
                self.take();
                Ok(neg(self.parse_unary()?))
            }
            Some('+') => {
                self.take();
                self.parse_unary()
            }
            _ => self.parse_power(),
        }
    }

    fn parse_power(&mut self) -> Result<Expr, String> {
        let base = self.parse_primary()?;
        if self.peek() == Some('^') {
            self.take();
            let exponent_expr = self.parse_unary()?;
            if let Some(exponent) = constant_value(&exponent_expr)? {
                return pow(base, exponent);
            }
            if matches!(base, Expr::Const(Const::E)) {
                return fn_expr(FunctionName::Exp, exponent_expr);
            }
            return Err(
                "Only constant exponents are supported; use exp(...) for exponentials.".into(),
            );
        }
        Ok(base)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let next = self
            .peek()
            .ok_or_else(|| "The expression ends early.".to_string())?;
        if next == '(' {
            self.take();
            let value = self.parse_sum()?;
            self.expect(')')?;
            return Ok(value);
        }
        if next.is_ascii_digit() || next == '.' {
            return self.parse_number();
        }
        if next.is_ascii_alphabetic() {
            return self.parse_identifier();
        }
        Err(format!("Unexpected “{next}”."))
    }

    fn parse_number(&mut self) -> Result<Expr, String> {
        let mut whole = String::new();
        let mut fraction = String::new();
        let mut seen_dot = false;
        while let Some(character) = self.peek() {
            if character.is_ascii_digit() {
                self.take();
                if seen_dot {
                    fraction.push(character);
                } else {
                    whole.push(character);
                }
            } else if character == '.' && !seen_dot {
                self.take();
                seen_dot = true;
            } else {
                break;
            }
        }
        if whole.is_empty() && fraction.is_empty() {
            return Err("Expected a number.".into());
        }
        let digits = format!(
            "{}{}",
            if whole.is_empty() {
                "0"
            } else {
                whole.as_str()
            },
            fraction
        );
        let numerator: i128 = digits
            .parse()
            .map_err(|_| "The number is too large.".to_string())?;
        let denominator = 10i128
            .checked_pow(fraction.chars().count() as u32)
            .ok_or_else(|| "The number is too large.".to_string())?;
        Ok(Expr::Num(Rat::new(numerator, denominator)?))
    }

    fn parse_identifier(&mut self) -> Result<Expr, String> {
        let mut name = String::new();
        while let Some(character) = self.peek() {
            if character.is_ascii_alphabetic() {
                self.take();
                name.push(character.to_ascii_lowercase());
            } else {
                break;
            }
        }
        if name.is_empty() {
            return Err("Expected a name.".into());
        }
        match name.as_str() {
            "x" => Ok(Expr::Var),
            "pi" => Ok(Expr::Const(Const::Pi)),
            "e" => Ok(Expr::Const(Const::E)),
            "sin" | "cos" | "tan" | "exp" | "ln" | "sqrt" | "sinh" | "cosh" => {
                let argument = if self.peek() == Some('(') {
                    self.take();
                    let inner = self.parse_sum()?;
                    self.expect(')')?;
                    inner
                } else {
                    self.parse_power()?
                };
                if name == "sqrt" {
                    return pow(argument, RAT_HALF);
                }
                let function = match name.as_str() {
                    "sin" => FunctionName::Sin,
                    "cos" => FunctionName::Cos,
                    "tan" => FunctionName::Tan,
                    "exp" => FunctionName::Exp,
                    "ln" => FunctionName::Ln,
                    "sinh" => FunctionName::Sinh,
                    "cosh" => FunctionName::Cosh,
                    _ => FunctionName::Sqrt,
                };
                fn_expr(function, argument)
            }
            _ => Err(format!(
                "Unknown name “{name}”. Use x, pi, e, sin, cos, tan, exp, ln, sqrt, sinh or cosh."
            )),
        }
    }
}

fn starts_primary(character: char) -> bool {
    character.is_ascii_digit()
        || character.is_ascii_alphabetic()
        || character == '.'
        || character == '('
}

pub fn parse_expression(source: &str) -> Result<Expr, String> {
    let text = normalize_source(source);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("Enter a function of x.".into());
    }
    let mut parser = Parser {
        source: trimmed.chars().collect(),
        index: 0,
    };
    let value = parser.parse_sum()?;
    if let Some(character) = parser.peek() {
        return Err(format!("Unexpected “{character}”."));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_implicit_multiplication() {
        let expression = parse_expression("2x").unwrap();
        assert_eq!(crate::expr::expr_plain(&expression), "2*x");
    }

    #[test]
    fn rejects_unknown_names() {
        assert!(parse_expression("sinh(x) + y")
            .unwrap_err()
            .contains("Unknown name"));
    }
}
