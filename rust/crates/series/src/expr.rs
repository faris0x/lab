//! A small symbolic expression engine for Taylor and Maclaurin series.

use mathcore::Rat;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Const {
    Pi,
    E,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FunctionName {
    Sin,
    Cos,
    Tan,
    Exp,
    Ln,
    Sqrt,
    Sinh,
    Cosh,
}

impl FunctionName {
    pub fn latex(self) -> &'static str {
        match self {
            FunctionName::Sin => "sin",
            FunctionName::Cos => "cos",
            FunctionName::Tan => "tan",
            FunctionName::Exp => "exp",
            FunctionName::Ln => "ln",
            FunctionName::Sqrt => "sqrt",
            FunctionName::Sinh => "sinh",
            FunctionName::Cosh => "cosh",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Num(Rat),
    Var,
    Const(Const),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Pow(Box<Expr>, Rat),
    Fn(FunctionName, Box<Expr>),
}

pub const RAT_HALF: Rat = Rat { n: 1, d: 2 };
pub const RAT_ONE: Rat = Rat { n: 1, d: 1 };

pub fn num(n: i64) -> Expr {
    Expr::Num(Rat::int(n))
}

pub fn rat_expr(value: Rat) -> Expr {
    Expr::Num(value)
}

pub fn zero() -> Expr {
    num(0)
}

pub fn one() -> Expr {
    num(1)
}

pub fn is_zero(value: &Expr) -> bool {
    matches!(value, Expr::Num(r) if r.is_zero())
}

pub fn is_one(value: &Expr) -> bool {
    matches!(value, Expr::Num(r) if r.n == r.d)
}

pub fn is_neg(value: &Expr) -> bool {
    match value {
        Expr::Num(r) => r.is_negative(),
        Expr::Neg(_) => true,
        Expr::Mul(left, right) => is_neg(left) != is_neg(right),
        Expr::Div(left, right) => is_neg(left) != is_neg(right),
        _ => false,
    }
}

pub fn rat_pow(base: Rat, exponent: i64) -> Result<Rat, String> {
    if exponent < 0 {
        if base.n == 0 {
            return Err("Division by zero.".into());
        }
        let flipped = Rat::new(base.d as i128, base.n as i128)?;
        return rat_pow(flipped, -exponent);
    }
    let mut power = exponent as u64;
    let mut factor = base;
    let mut result = Rat::ONE;
    while power > 0 {
        if power & 1 == 1 {
            result = result.mul(factor)?;
        }
        factor = factor.mul(factor)?;
        power >>= 1;
    }
    Ok(result)
}

fn integer_sqrt(value: i128) -> Option<i128> {
    if value < 0 {
        return None;
    }
    if value < 2 {
        return Some(value);
    }
    let mut low = 1i128;
    let mut high = value;
    while low <= high {
        let mid = (low + high) / 2;
        let square = mid.checked_mul(mid)?;
        if square == value {
            return Some(mid);
        }
        if square < value {
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }
    None
}

pub fn exact_sqrt(value: Rat) -> Option<Rat> {
    if value.n < 0 {
        return None;
    }
    let n = integer_sqrt(value.n as i128)?;
    let d = integer_sqrt(value.d as i128)?;
    Rat::new(n, d).ok()
}

pub fn add(left: Expr, right: Expr) -> Result<Expr, String> {
    if let (Expr::Num(a), Expr::Num(b)) = (&left, &right) {
        return Ok(Expr::Num(a.add(*b)?));
    }
    if is_zero(&left) {
        return Ok(right);
    }
    if is_zero(&right) {
        return Ok(left);
    }
    if left == right {
        return mul(num(2), left);
    }
    Ok(Expr::Add(Box::new(left), Box::new(right)))
}

pub fn sub(left: Expr, right: Expr) -> Result<Expr, String> {
    if let (Expr::Num(a), Expr::Num(b)) = (&left, &right) {
        return Ok(Expr::Num(a.sub(*b)?));
    }
    if is_zero(&right) {
        return Ok(left);
    }
    if is_zero(&left) {
        return Ok(neg(right));
    }
    if left == right {
        return Ok(zero());
    }
    Ok(Expr::Sub(Box::new(left), Box::new(right)))
}

pub fn neg(child: Expr) -> Expr {
    match child {
        Expr::Num(value) => Expr::Num(value.neg()),
        Expr::Neg(inner) => *inner,
        Expr::Mul(left, right) => match *left {
            Expr::Num(value) => Expr::Mul(Box::new(Expr::Num(value.neg())), right),
            other => Expr::Neg(Box::new(Expr::Mul(Box::new(other), right))),
        },
        other => Expr::Neg(Box::new(other)),
    }
}

pub fn mul(left: Expr, right: Expr) -> Result<Expr, String> {
    if is_zero(&left) || is_zero(&right) {
        return Ok(zero());
    }
    if is_one(&left) {
        return Ok(right);
    }
    if is_one(&right) {
        return Ok(left);
    }
    if let (Expr::Num(a), Expr::Num(b)) = (&left, &right) {
        return Ok(Expr::Num(a.mul(*b)?));
    }
    if matches!(right, Expr::Num(_)) && !matches!(left, Expr::Num(_)) {
        return mul(right, left);
    }
    if let Expr::Neg(inner) = left {
        return Ok(neg(mul(*inner, right)?));
    }
    if let Expr::Neg(inner) = right {
        return Ok(neg(mul(left, *inner)?));
    }
    if let (Expr::Num(value), Expr::Div(numerator, denominator)) = (&left, &right) {
        return div(
            mul(Expr::Num(*value), *numerator.clone())?,
            *denominator.clone(),
        );
    }
    if let (Expr::Num(value), Expr::Mul(inner_left, inner_right)) = (&left, &right) {
        if let Expr::Num(_) = inner_left.as_ref() {
            return mul(
                mul(Expr::Num(*value), *inner_left.clone())?,
                *inner_right.clone(),
            );
        }
    }
    if let (Expr::Pow(left_base, left_exp), Expr::Pow(right_base, right_exp)) = (&left, &right) {
        if left_base == right_base {
            return pow(*left_base.clone(), left_exp.add(*right_exp)?);
        }
    }
    if left == right {
        return pow(left, Rat::int(2));
    }
    if let Expr::Pow(base, exponent) = &left {
        if base.as_ref() == &right {
            return pow(*base.clone(), exponent.add(RAT_ONE)?);
        }
    }
    if let Expr::Pow(base, exponent) = &right {
        if base.as_ref() == &left {
            return pow(*base.clone(), exponent.add(RAT_ONE)?);
        }
    }
    Ok(Expr::Mul(Box::new(left), Box::new(right)))
}

pub fn div(left: Expr, right: Expr) -> Result<Expr, String> {
    if is_zero(&right) {
        return Err("Division by zero.".into());
    }
    if is_zero(&left) {
        return Ok(zero());
    }
    if is_one(&right) {
        return Ok(left);
    }
    if left == right {
        return Ok(one());
    }
    if let (Expr::Num(a), Expr::Num(b)) = (&left, &right) {
        return Ok(Expr::Num(a.div(*b)?));
    }
    if let Expr::Neg(inner) = right {
        return div(neg(left), *inner);
    }
    if let Expr::Neg(inner) = left {
        return Ok(neg(div(*inner, right)?));
    }
    if let (Expr::Num(value), Expr::Mul(inner_left, inner_right)) = (&left, &right) {
        if let Expr::Num(factor) = inner_left.as_ref() {
            return mul(
                Expr::Num(value.div(*factor)?),
                div(one(), *inner_right.clone())?,
            );
        }
    }
    if let Expr::Div(numerator, denominator) = left.clone() {
        return div(*numerator, mul(*denominator, right)?);
    }
    if let Expr::Div(numerator, denominator) = right.clone() {
        return div(mul(left, *denominator)?, *numerator);
    }
    if let Expr::Mul(factor_left, factor_right) = left.clone() {
        let cancel = |factor: &Expr, other: &Expr| -> Result<Option<Expr>, String> {
            if factor == &right {
                return Ok(Some(other.clone()));
            }
            if let (Expr::Pow(factor_base, factor_exp), Expr::Pow(right_base, right_exp)) =
                (factor, &right)
            {
                if factor_base == right_base {
                    return Ok(Some(mul(
                        other.clone(),
                        pow(*factor_base.clone(), factor_exp.sub(*right_exp)?)?,
                    )?));
                }
            }
            if let Expr::Pow(factor_base, factor_exp) = factor {
                if factor_base.as_ref() == &right {
                    return Ok(Some(mul(
                        other.clone(),
                        pow(*factor_base.clone(), factor_exp.sub(RAT_ONE)?)?,
                    )?));
                }
            }
            if let Expr::Pow(right_base, right_exp) = &right {
                if factor == right_base.as_ref() {
                    return Ok(Some(mul(
                        other.clone(),
                        pow(*right_base.clone(), Rat::new(1, 1)?.sub(*right_exp)?)?,
                    )?));
                }
            }
            Ok(None)
        };
        if let Some(result) = cancel(&factor_left, &factor_right)? {
            return Ok(result);
        }
        if let Some(result) = cancel(&factor_right, &factor_left)? {
            return Ok(result);
        }
    }
    if let (Expr::Pow(left_base, left_exp), Expr::Pow(right_base, right_exp)) = (&left, &right) {
        if left_base == right_base {
            return pow(*left_base.clone(), left_exp.sub(*right_exp)?);
        }
    }
    if let Expr::Pow(base, exponent) = &left {
        if base.as_ref() == &right {
            return pow(*base.clone(), exponent.sub(RAT_ONE)?);
        }
    }
    if let Expr::Pow(base, exponent) = &right {
        if base.as_ref() == &left {
            return pow(*base.clone(), Rat::ONE.sub(*exponent)?);
        }
    }
    if let Expr::Num(value) = &right {
        if let Expr::Mul(inner_left, inner_right) = &left {
            if let Expr::Num(factor) = inner_left.as_ref() {
                return mul(Expr::Num(factor.div(*value)?), *inner_right.clone());
            }
        }
        return mul(Expr::Num(Rat::ONE.div(*value)?), left);
    }
    Ok(Expr::Div(Box::new(left), Box::new(right)))
}

pub fn pow(base: Expr, exponent: Rat) -> Result<Expr, String> {
    if exponent.n == 0 {
        return Ok(one());
    }
    if exponent.n == exponent.d {
        return Ok(base);
    }
    if is_one(&base) {
        return Ok(one());
    }
    if is_zero(&base) {
        if exponent.n < 0 {
            return Err("Division by zero.".into());
        }
        return Ok(zero());
    }
    if let Expr::Num(value) = &base {
        if exponent.d == 1 {
            return Ok(Expr::Num(rat_pow(*value, exponent.n)?));
        }
        if exponent.n == 1 && exponent.d == 2 {
            if let Some(root) = exact_sqrt(*value) {
                return Ok(Expr::Num(root));
            }
            if value.n < 0 {
                return Err("The square root of a negative number is not real.".into());
            }
            return Ok(Expr::Pow(Box::new(base), exponent));
        }
    }
    if let Expr::Pow(inner, inner_exp) = &base {
        return pow(*inner.clone(), inner_exp.mul(exponent)?);
    }
    if let Expr::Mul(left, right) = &base {
        if exponent.d == 1 {
            return mul(
                pow(*left.clone(), exponent)?,
                pow(*right.clone(), exponent)?,
            );
        }
    }
    if let Expr::Fn(FunctionName::Sqrt, inner) = &base {
        if exponent.d == 1 && exponent.n % 2 == 0 {
            return pow(*inner.clone(), Rat::int(exponent.n / 2));
        }
    }
    Ok(Expr::Pow(Box::new(base), exponent))
}

pub fn fn_expr(name: FunctionName, arg: Expr) -> Result<Expr, String> {
    if let Expr::Num(value) = &arg {
        match name {
            FunctionName::Sin if value.is_zero() => return Ok(zero()),
            FunctionName::Cos if value.is_zero() => return Ok(one()),
            FunctionName::Tan if value.is_zero() => return Ok(zero()),
            FunctionName::Exp if value.is_zero() => return Ok(one()),
            FunctionName::Sinh if value.is_zero() => return Ok(zero()),
            FunctionName::Cosh if value.is_zero() => return Ok(one()),
            FunctionName::Ln => {
                if value.n <= 0 {
                    return Err("The logarithm is only real for positive values.".into());
                }
                if value.n == value.d {
                    return Ok(zero());
                }
            }
            FunctionName::Sqrt => {
                if value.n < 0 {
                    return Err("The square root of a negative number is not real.".into());
                }
                if let Some(root) = exact_sqrt(*value) {
                    return Ok(Expr::Num(root));
                }
            }
            _ => {}
        }
    }
    Ok(Expr::Fn(name, Box::new(arg)))
}

pub fn substitute(value: &Expr, center: Rat) -> Result<Expr, String> {
    match value {
        Expr::Num(_) | Expr::Const(_) => Ok(value.clone()),
        Expr::Var => Ok(Expr::Num(center)),
        Expr::Add(left, right) => add(substitute(left, center)?, substitute(right, center)?),
        Expr::Sub(left, right) => sub(substitute(left, center)?, substitute(right, center)?),
        Expr::Mul(left, right) => mul(substitute(left, center)?, substitute(right, center)?),
        Expr::Div(left, right) => div(substitute(left, center)?, substitute(right, center)?),
        Expr::Neg(child) => Ok(neg(substitute(child, center)?)),
        Expr::Pow(base, exponent) => pow(substitute(base, center)?, *exponent),
        Expr::Fn(name, arg) => fn_expr(*name, substitute(arg, center)?),
    }
}

pub fn constant_value(value: &Expr) -> Result<Option<Rat>, String> {
    match value {
        Expr::Num(number) => Ok(Some(*number)),
        Expr::Neg(child) => Ok(constant_value(child)?.map(Rat::neg)),
        Expr::Add(left, right) => match (constant_value(left)?, constant_value(right)?) {
            (Some(a), Some(b)) => Ok(Some(a.add(b)?)),
            _ => Ok(None),
        },
        Expr::Sub(left, right) => match (constant_value(left)?, constant_value(right)?) {
            (Some(a), Some(b)) => Ok(Some(a.sub(b)?)),
            _ => Ok(None),
        },
        Expr::Mul(left, right) => match (constant_value(left)?, constant_value(right)?) {
            (Some(a), Some(b)) => Ok(Some(a.mul(b)?)),
            _ => Ok(None),
        },
        Expr::Div(left, right) => match (constant_value(left)?, constant_value(right)?) {
            (Some(a), Some(b)) => Ok(Some(a.div(b)?)),
            _ => Ok(None),
        },
        Expr::Pow(base, exponent) => {
            let Some(base) = constant_value(base)? else {
                return Ok(None);
            };
            if exponent.d == 1 {
                return Ok(Some(rat_pow(base, exponent.n)?));
            }
            if exponent.n == 1 && exponent.d == 2 {
                return Ok(exact_sqrt(base));
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

pub fn derivative(value: &Expr) -> Result<Expr, String> {
    match value {
        Expr::Num(_) | Expr::Const(_) => Ok(zero()),
        Expr::Var => Ok(one()),
        Expr::Add(left, right) => add(derivative(left)?, derivative(right)?),
        Expr::Sub(left, right) => sub(derivative(left)?, derivative(right)?),
        Expr::Mul(left, right) => add(
            mul(derivative(left)?, *right.clone())?,
            mul(*left.clone(), derivative(right)?)?,
        ),
        Expr::Div(left, right) => {
            let numerator = sub(
                mul(derivative(left)?, *right.clone())?,
                mul(*left.clone(), derivative(right)?)?,
            )?;
            div(numerator, pow(*right.clone(), Rat::int(2))?)
        }
        Expr::Neg(child) => Ok(neg(derivative(child)?)),
        Expr::Pow(base, exponent) => {
            let inner = derivative(base)?;
            let lowered = exponent.sub(RAT_ONE)?;
            mul(
                Expr::Num(*exponent),
                mul(pow(*base.clone(), lowered)?, inner)?,
            )
        }
        Expr::Fn(name, arg) => {
            let inner = derivative(arg)?;
            match name {
                FunctionName::Sin => mul(Expr::Fn(FunctionName::Cos, arg.clone()), inner),
                FunctionName::Cos => Ok(neg(mul(Expr::Fn(FunctionName::Sin, arg.clone()), inner)?)),
                FunctionName::Tan => div(
                    inner,
                    pow(Expr::Fn(FunctionName::Cos, arg.clone()), Rat::int(2))?,
                ),
                FunctionName::Exp => mul(Expr::Fn(FunctionName::Exp, arg.clone()), inner),
                FunctionName::Ln => div(inner, *arg.clone()),
                FunctionName::Sqrt => div(
                    inner,
                    mul(num(2), Expr::Fn(FunctionName::Sqrt, arg.clone()))?,
                ),
                FunctionName::Sinh => mul(Expr::Fn(FunctionName::Cosh, arg.clone()), inner),
                FunctionName::Cosh => mul(Expr::Fn(FunctionName::Sinh, arg.clone()), inner),
            }
        }
    }
}

pub fn node_count(value: &Expr) -> usize {
    match value {
        Expr::Num(_) | Expr::Var | Expr::Const(_) => 1,
        Expr::Neg(child) => 1 + node_count(child),
        Expr::Add(left, right)
        | Expr::Sub(left, right)
        | Expr::Mul(left, right)
        | Expr::Div(left, right) => 1 + node_count(left) + node_count(right),
        Expr::Pow(base, _) => 1 + node_count(base),
        Expr::Fn(_, arg) => 1 + node_count(arg),
    }
}

pub fn factorial(value: usize) -> Rat {
    let mut result = 1i128;
    for index in 2..=value {
        result *= index as i128;
    }
    Rat::new(result, 1).unwrap_or(Rat::ONE)
}

pub fn rat_latex(value: Rat) -> String {
    value.latex()
}

pub fn rat_plain(value: Rat) -> String {
    value.plain()
}

fn wrap_latex(value: &Expr, inner: String) -> String {
    match value {
        Expr::Add(_, _) | Expr::Sub(_, _) => format!("\\left({inner}\\right)"),
        _ => inner,
    }
}

pub fn expr_latex(value: &Expr) -> String {
    match value {
        Expr::Num(number) => rat_latex(*number),
        Expr::Var => "x".into(),
        Expr::Const(Const::Pi) => "\\pi".into(),
        Expr::Const(Const::E) => "e".into(),
        Expr::Add(left, right) => format!("{} + {}", expr_latex(left), expr_latex(right)),
        Expr::Sub(left, right) => format!("{} - {}", expr_latex(left), expr_latex(right)),
        Expr::Mul(left, right) => {
            if let Expr::Num(number) = left.as_ref() {
                if !matches!(right.as_ref(), Expr::Num(_)) {
                    return format!(
                        "{}{}",
                        rat_latex(*number),
                        wrap_latex(right, expr_latex(right))
                    );
                }
            }
            format!(
                "{} \\cdot {}",
                wrap_latex(left, expr_latex(left)),
                wrap_latex(right, expr_latex(right))
            )
        }
        Expr::Div(left, right) => {
            format!("\\frac{{{}}}{{{}}}", expr_latex(left), expr_latex(right))
        }
        Expr::Neg(child) => format!("-{}", wrap_latex(child, expr_latex(child))),
        Expr::Pow(base, exponent) => {
            if exponent.n == 1 && exponent.d == 2 {
                return format!("\\sqrt{{{}}}", expr_latex(base));
            }
            if exponent.n == -1 && exponent.d == 1 {
                return format!("\\frac{{1}}{{{}}}", expr_latex(base));
            }
            let base_text = match base.as_ref() {
                Expr::Add(_, _)
                | Expr::Sub(_, _)
                | Expr::Mul(_, _)
                | Expr::Div(_, _)
                | Expr::Pow(_, _) => format!("\\left({}\\right)", expr_latex(base)),
                _ => expr_latex(base),
            };
            format!("{}^{{{}}}", base_text, rat_latex(*exponent))
        }
        Expr::Fn(name, arg) => match name {
            FunctionName::Exp => format!("e^{{{}}}", expr_latex(arg)),
            FunctionName::Sqrt => format!("\\sqrt{{{}}}", expr_latex(arg)),
            FunctionName::Ln => format!("\\ln\\left({}\\right)", expr_latex(arg)),
            other => format!("\\{}\\left({}\\right)", other.latex(), expr_latex(arg)),
        },
    }
}

fn wrap_plain(value: &Expr, inner: String) -> String {
    match value {
        Expr::Add(_, _) | Expr::Sub(_, _) | Expr::Neg(_) | Expr::Mul(_, _) | Expr::Div(_, _) => {
            format!("({inner})")
        }
        _ => inner,
    }
}

pub fn expr_plain(value: &Expr) -> String {
    match value {
        Expr::Num(number) => rat_plain(*number),
        Expr::Var => "x".into(),
        Expr::Const(Const::Pi) => "pi".into(),
        Expr::Const(Const::E) => "e".into(),
        Expr::Add(left, right) => format!("{} + {}", expr_plain(left), expr_plain(right)),
        Expr::Sub(left, right) => format!("{} - {}", expr_plain(left), expr_plain(right)),
        Expr::Mul(left, right) => format!(
            "{}*{}",
            wrap_plain(left, expr_plain(left)),
            wrap_plain(right, expr_plain(right))
        ),
        Expr::Div(left, right) => format!(
            "{}/{}",
            wrap_plain(left, expr_plain(left)),
            wrap_plain(right, expr_plain(right))
        ),
        Expr::Neg(child) => format!("-{}", wrap_plain(child, expr_plain(child))),
        Expr::Pow(base, exponent) => format!(
            "{}^{}",
            wrap_plain(base, expr_plain(base)),
            rat_plain(*exponent)
        ),
        Expr::Fn(name, arg) => format!("{}({})", name.latex(), expr_plain(arg)),
    }
}

pub fn shift_latex(center: Rat) -> String {
    if center.is_zero() {
        return "x".into();
    }
    let magnitude = rat_latex(center.abs());
    if center.n > 0 {
        format!("\\left(x - {magnitude}\\right)")
    } else {
        format!("\\left(x + {magnitude}\\right)")
    }
}

pub fn shift_plain(center: Rat) -> String {
    if center.is_zero() {
        return "x".into();
    }
    let magnitude = rat_plain(center.abs());
    if center.n > 0 {
        format!("(x - {magnitude})")
    } else {
        format!("(x + {magnitude})")
    }
}

pub fn factor_latex(center: Rat, k: usize) -> String {
    if k == 0 {
        return String::new();
    }
    let shift = shift_latex(center);
    if k == 1 {
        shift
    } else {
        format!("{shift}^{{{k}}}")
    }
}

fn term_body_latex(coefficient: &Expr, center: Rat, k: usize) -> String {
    let factor = factor_latex(center, k);
    if k == 0 {
        return expr_latex(coefficient);
    }
    if is_one(coefficient) {
        return factor;
    }
    match coefficient {
        Expr::Num(number) => format!("{}{}", rat_latex(*number), factor),
        _ => format!("\\left({}\\right){}", expr_latex(coefficient), factor),
    }
}

pub fn term_latex(coefficient: &Expr, center: Rat, k: usize) -> String {
    if is_zero(coefficient) {
        return "0".into();
    }
    if is_neg(coefficient) {
        return format!("-{}", term_body_latex(&neg(coefficient.clone()), center, k));
    }
    term_body_latex(coefficient, center, k)
}

pub fn series_latex(terms: &[(Expr, usize)], center: Rat) -> String {
    let visible: Vec<&(Expr, usize)> = terms
        .iter()
        .filter(|(coefficient, _)| !is_zero(coefficient))
        .collect();
    if visible.is_empty() {
        return "0".into();
    }
    let mut output = String::new();
    for (index, (coefficient, k)) in visible.iter().enumerate() {
        let negative = is_neg(coefficient);
        let positive = if negative {
            neg((*coefficient).clone())
        } else {
            (*coefficient).clone()
        };
        let body = term_body_latex(&positive, center, *k);
        if index == 0 {
            if negative {
                output.push('-');
            }
            output.push_str(&body);
        } else if negative {
            output.push_str(" - ");
            output.push_str(&body);
        } else {
            output.push_str(" + ");
            output.push_str(&body);
        }
    }
    output
}

pub fn series_plain(terms: &[(Expr, usize)], center: Rat) -> String {
    let visible: Vec<&(Expr, usize)> = terms
        .iter()
        .filter(|(coefficient, _)| !is_zero(coefficient))
        .collect();
    if visible.is_empty() {
        return "0".into();
    }
    let mut output = String::new();
    for (index, (coefficient, k)) in visible.iter().enumerate() {
        let negative = is_neg(coefficient);
        let positive = if negative {
            neg((*coefficient).clone())
        } else {
            (*coefficient).clone()
        };
        let factor = if *k == 0 {
            String::new()
        } else if *k == 1 {
            shift_plain(center)
        } else {
            format!("{}^{}", shift_plain(center), k)
        };
        let body = if *k == 0 {
            expr_plain(&positive)
        } else if is_one(&positive) {
            factor
        } else {
            match &positive {
                Expr::Num(number) => format!("{}{}", rat_plain(*number), factor),
                _ => format!("({}){}", expr_plain(&positive), factor),
            }
        };
        if index == 0 {
            if negative {
                output.push('-');
            }
            output.push_str(&body);
        } else if negative {
            output.push_str(" - ");
            output.push_str(&body);
        } else {
            output.push_str(" + ");
            output.push_str(&body);
        }
    }
    output
}
