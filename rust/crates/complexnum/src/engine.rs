//! Exact complex arithmetic plus numeric polar/root work for the Argand tool.

use mathcore::Rat;
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug)]
pub struct Exact {
    pub re: Rat,
    pub im: Rat,
}

impl Exact {
    pub fn new(re: Rat, im: Rat) -> Self {
        Exact { re, im }
    }
    pub fn one() -> Self {
        Exact {
            re: Rat::ONE,
            im: Rat::ZERO,
        }
    }
    pub fn add(self, o: Self) -> Result<Self, String> {
        Ok(Exact {
            re: self.re.add(o.re)?,
            im: self.im.add(o.im)?,
        })
    }
    pub fn sub(self, o: Self) -> Result<Self, String> {
        Ok(Exact {
            re: self.re.sub(o.re)?,
            im: self.im.sub(o.im)?,
        })
    }
    pub fn mul(self, o: Self) -> Result<Self, String> {
        Ok(Exact {
            re: self.re.mul(o.re)?.sub(self.im.mul(o.im)?)?,
            im: self.re.mul(o.im)?.add(self.im.mul(o.re)?)?,
        })
    }
    pub fn div(self, o: Self) -> Result<Self, String> {
        let den = o.re.mul(o.re)?.add(o.im.mul(o.im)?)?;
        if den.is_zero() {
            return Err("division by zero".into());
        }
        Ok(Exact {
            re: self.re.mul(o.re)?.add(self.im.mul(o.im)?)?.div(den)?,
            im: self.im.mul(o.re)?.sub(self.re.mul(o.im)?)?.div(den)?,
        })
    }
    pub fn approx(self) -> Approx {
        Approx {
            re: self.re.n as f64 / self.re.d as f64,
            im: self.im.n as f64 / self.im.d as f64,
        }
    }
    pub fn is_integral(self) -> bool {
        self.re.d == 1 && self.im.d == 1
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Approx {
    pub re: f64,
    pub im: f64,
}

/// Round to at most four decimal places, then remove trailing zeroes.
pub fn fmt4(x: f64) -> String {
    if !x.is_finite() {
        return "0".to_string();
    }
    let mut s = format!("{:.4}", x);
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    if s == "-0" || s == "0" {
        "0".to_string()
    } else {
        s
    }
}

pub fn exact_plain(z: Exact) -> String {
    let re = z.re.plain();
    if z.im.is_zero() {
        return re;
    }
    let im_abs = z.im.abs();
    let im_part = if im_abs == Rat::ONE {
        "i".to_string()
    } else {
        format!("{}i", im_abs.plain())
    };
    if z.re.is_zero() {
        if z.im.is_negative() {
            format!("-{im_part}")
        } else {
            im_part
        }
    } else if z.im.is_negative() {
        format!("{re} - {im_part}")
    } else {
        format!("{re} + {im_part}")
    }
}

pub fn exact_latex(z: Exact) -> String {
    let re = z.re.latex();
    if z.im.is_zero() {
        return re;
    }
    let im_abs = z.im.abs();
    let im_part = if im_abs == Rat::ONE {
        "i".to_string()
    } else {
        format!("{}i", im_abs.latex())
    };
    if z.re.is_zero() {
        if z.im.is_negative() {
            format!("-{im_part}")
        } else {
            im_part
        }
    } else if z.im.is_negative() {
        format!("{re} - {im_part}")
    } else {
        format!("{re} + {im_part}")
    }
}

pub fn approx_latex(z: Approx) -> String {
    let re = fmt4(z.re);
    if z.im.abs() < 0.00005 {
        return re;
    }
    let abs = z.im.abs();
    let im_part = if (abs - 1.0).abs() < 0.00005 {
        "i".to_string()
    } else {
        format!("{}i", fmt4(abs))
    };
    if z.re.abs() < 0.00005 {
        if z.im < 0.0 {
            format!("-{im_part}")
        } else {
            im_part
        }
    } else if z.im < 0.0 {
        format!("{re} - {im_part}")
    } else {
        format!("{re} + {im_part}")
    }
}

fn terminating_decimal(r: Rat) -> Option<String> {
    if r.d == 1 {
        return Some(r.n.to_string());
    }
    let mut d = r.d;
    let mut twos = 0u32;
    let mut fives = 0u32;
    while d % 2 == 0 {
        d /= 2;
        twos += 1;
    }
    while d % 5 == 0 {
        d /= 5;
        fives += 1;
    }
    if d != 1 {
        return None;
    }
    let places = twos.max(fives);
    let scale = 10i128.checked_pow(places)?;
    let scaled = (r.n as i128) * scale / (r.d as i128);
    if places == 0 {
        return Some(scaled.to_string());
    }
    let negative = scaled < 0;
    let mut digits = scaled.abs().to_string();
    while digits.len() <= places as usize {
        digits.insert(0, '0');
    }
    let split = digits.len() - places as usize;
    let mut out = format!("{}.{}", &digits[..split], &digits[split..]);
    while out.ends_with('0') {
        out.pop();
    }
    if out.ends_with('.') {
        out.pop();
    }
    if negative {
        out.insert(0, '-');
    }
    Some(out)
}

fn terminating_complex_decimal(z: Exact) -> Option<String> {
    let re = terminating_decimal(z.re)?;
    if z.im.is_zero() {
        return Some(re);
    }
    let im_abs = terminating_decimal(z.im.abs())?;
    let im_part = if im_abs == "1" {
        "i".to_string()
    } else {
        format!("{im_abs}i")
    };
    if z.re.is_zero() {
        if z.im.is_negative() {
            Some(format!("-{im_part}"))
        } else {
            Some(im_part)
        }
    } else if z.im.is_negative() {
        Some(format!("{re} - {im_part}"))
    } else {
        Some(format!("{re} + {im_part}"))
    }
}

fn isqrt(n: i64) -> Option<i64> {
    if n < 0 {
        return None;
    }
    let r = (n as f64).sqrt().round() as i64;
    if r * r == n {
        Some(r)
    } else {
        None
    }
}

fn sqrt_exact(r: Rat) -> Option<Rat> {
    let n = isqrt(r.n)?;
    let d = isqrt(r.d)?;
    Rat::new(n as i128, d as i128).ok()
}

fn modulus_exact_latex(z: Exact) -> String {
    let r2 = match z
        .re
        .mul(z.re)
        .and_then(|a| z.im.mul(z.im).and_then(|b| a.add(b)))
    {
        Ok(v) => v,
        Err(_) => return "0".into(),
    };
    if r2.is_zero() {
        return "0".into();
    }
    if let Some(r) = sqrt_exact(r2) {
        r.latex()
    } else {
        format!("\\sqrt{{{}}}", r2.latex())
    }
}

fn modulus_exact_plain(z: Exact) -> String {
    let r2 = match z
        .re
        .mul(z.re)
        .and_then(|a| z.im.mul(z.im).and_then(|b| a.add(b)))
    {
        Ok(v) => v,
        Err(_) => return "0".into(),
    };
    if r2.is_zero() {
        return "0".into();
    }
    if let Some(r) = sqrt_exact(r2) {
        r.plain()
    } else {
        format!("sqrt({})", r2.plain())
    }
}

fn pi_fraction(theta: f64) -> Option<(i64, i64)> {
    for d in [1i64, 2, 3, 4, 6, 12] {
        let k = (theta * d as f64 / PI).round() as i64;
        if (theta - k as f64 * PI / d as f64).abs() < 1e-9 {
            let g = gcd(k, d);
            return Some((k / g, d / g));
        }
    }
    None
}

fn gcd(mut a: i64, mut b: i64) -> i64 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    if a == 0 {
        1
    } else {
        a
    }
}

fn pi_fraction_up_to(theta: f64, max_den: i64) -> Option<(i64, i64)> {
    if theta.abs() < 1e-12 {
        return Some((0, 1));
    }
    for d in 1..=max_den.max(1) {
        let k = (theta * d as f64 / PI).round() as i64;
        if (theta - k as f64 * PI / d as f64).abs() < 1e-10 {
            let g = gcd(k, d);
            return Some((k / g, d / g));
        }
    }
    None
}

fn pi_fraction_latex(k: i64, d: i64) -> String {
    if k == 0 {
        return "0".into();
    }
    let sign = if k < 0 { "-" } else { "" };
    let k = k.abs();
    if d == 1 {
        if k == 1 {
            format!("{sign}\\pi")
        } else {
            format!("{sign}{k}\\pi")
        }
    } else if k == 1 {
        format!("{sign}\\frac{{\\pi}}{{{d}}}")
    } else {
        format!("{sign}\\frac{{{k}\\pi}}{{{d}}}")
    }
}

fn pi_fraction_plain(k: i64, d: i64) -> String {
    if k == 0 {
        return "0".into();
    }
    let sign = if k < 0 { "-" } else { "" };
    let k = k.abs();
    if d == 1 {
        if k == 1 {
            format!("{sign}π")
        } else {
            format!("{sign}{k}π")
        }
    } else if k == 1 {
        format!("{sign}π/{d}")
    } else {
        format!("{sign}{k}π/{d}")
    }
}

fn argument_exact_latex(z: Exact, theta: f64) -> String {
    if let Some((k, d)) = pi_fraction(theta) {
        pi_fraction_latex(k, d)
    } else {
        format!(
            "\\operatorname{{atan2}}\\left({}, {}\\right)",
            z.im.latex(),
            z.re.latex()
        )
    }
}

fn argument_exact_plain(z: Exact, theta: f64) -> String {
    if let Some((k, d)) = pi_fraction(theta) {
        pi_fraction_plain(k, d)
    } else {
        format!("atan2({}, {})", z.im.plain(), z.re.plain())
    }
}

pub struct Polar {
    pub r: f64,
    pub theta: f64,
    pub mod_exact_latex: String,
    pub mod_exact_plain: String,
    pub mod_approx: String,
    pub mod_is_rational: bool,
    pub arg_exact_latex: String,
    pub arg_exact_plain: String,
    pub arg_approx: String,
    pub arg_is_zero: bool,
    pub degrees: String,
}

impl Polar {
    pub fn from(z: Exact) -> Self {
        let za = z.approx();
        let r = za.re.hypot(za.im);
        let theta = za.im.atan2(za.re);
        let (arg_exact_latex, arg_exact_plain, arg_approx, degrees) = if r.abs() < 0.00005 {
            (
                "\\text{undefined}".to_string(),
                "undefined".to_string(),
                "undefined".to_string(),
                "undefined".to_string(),
            )
        } else {
            (
                argument_exact_latex(z, theta),
                argument_exact_plain(z, theta),
                fmt4(theta),
                fmt4(theta.to_degrees()),
            )
        };
        let r2 =
            z.re.mul(z.re)
                .and_then(|a| z.im.mul(z.im).and_then(|b| a.add(b)))
                .unwrap_or(Rat::ZERO);
        Polar {
            r,
            theta,
            mod_exact_latex: modulus_exact_latex(z),
            mod_exact_plain: modulus_exact_plain(z),
            mod_approx: fmt4(r),
            mod_is_rational: sqrt_exact(r2).is_some(),
            arg_exact_latex,
            arg_exact_plain,
            arg_approx,
            arg_is_zero: theta.abs() < 1e-9,
            degrees,
        }
    }

    pub fn form_exact_latex(&self) -> String {
        if self.r.abs() < 0.00005 {
            return "0".into();
        }
        format!(
            "{}\\left(\\cos {} + i\\sin {}\\right)",
            self.mod_exact_latex, self.arg_exact_latex, self.arg_exact_latex
        )
    }

    pub fn form_approx_latex(&self) -> String {
        if self.r.abs() < 0.00005 {
            return "0".into();
        }
        format!(
            "{}\\left(\\cos {} + i\\sin {}\\right)",
            self.mod_approx, self.arg_approx, self.arg_approx
        )
    }

    pub fn exp_exact_latex(&self) -> String {
        if self.r.abs() < 0.00005 {
            return "0".into();
        }
        format!("{}e^{{i{}}}", self.mod_exact_latex, self.arg_exact_latex)
    }
}

#[derive(Clone, Debug)]
pub struct Step {
    pub description: String,
    pub description_latex: String,
}

#[derive(Clone, Debug)]
pub struct Point {
    pub label: String,
    pub label_latex: String,
    pub x: f64,
    pub y: f64,
    pub kind: &'static str,
}

#[derive(Clone, Debug)]
pub struct Circle {
    pub radius: f64,
}

#[derive(Clone, Debug, Default)]
pub struct Argand {
    pub points: Vec<Point>,
    pub circles: Vec<Circle>,
}

pub struct Outcome {
    pub operation: &'static str,
    pub z: Exact,
    pub w: Option<Exact>,
    pub steps: Vec<Step>,
    pub summary: Vec<String>,
    pub summary_latex: Vec<String>,
    pub lines: Vec<String>,
    pub argand: Argand,
}

fn point(label: &str, latex: &str, z: Approx, kind: &'static str) -> Point {
    Point {
        label: label.to_string(),
        label_latex: latex.to_string(),
        x: z.re,
        y: z.im,
        kind,
    }
}

pub fn arithmetic(z: Exact, w: Exact, op: &str) -> Result<Outcome, String> {
    let result = match op {
        "+" => z.add(w)?,
        "-" => z.sub(w)?,
        "*" => z.mul(w)?,
        "/" => z.div(w)?,
        _ => return Err("choose an arithmetic operation".into()),
    };
    let label_plain = match op {
        "+" => "z + w",
        "-" => "z - w",
        "*" => "zw",
        "/" => "z/w",
        _ => "result",
    };
    let label_latex = match op {
        "+" => "z + w",
        "-" => "z - w",
        "*" => "zw",
        "/" => "\\frac{z}{w}",
        _ => "result",
    };

    let mut steps = Vec::new();
    match op {
        "+" | "-" => {
            let verb = if op == "+" { "Add" } else { "Subtract" };
            let sign = if op == "+" { "+" } else { "-" };
            steps.push(Step {
                description: format!(
                    "{verb} the real parts: {} {sign} {} = {}.",
                    z.re.plain(),
                    w.re.plain(),
                    result.re.plain()
                ),
                description_latex: format!(
                    "{verb} the real parts: ${} {sign} {} = {}$.",
                    z.re.latex(),
                    w.re.latex(),
                    result.re.latex()
                ),
            });
            steps.push(Step {
                description: format!(
                    "{verb} the imaginary parts: {} {sign} {} = {}.",
                    z.im.plain(),
                    w.im.plain(),
                    result.im.plain()
                ),
                description_latex: format!(
                    "{verb} the imaginary parts: ${} {sign} {} = {}$.",
                    z.im.latex(),
                    w.im.latex(),
                    result.im.latex()
                ),
            });
        }
        "*" => {
            steps.push(Step {
                description: "Multiply out (a + bi)(c + di) = (ac - bd) + (ad + bc)i.".into(),
                description_latex: "Multiply out $(a + bi)(c + di) = (ac - bd) + (ad + bc)i$."
                    .into(),
            });
        }
        "/" => {
            let conj = Exact {
                re: w.re,
                im: w.im.neg(),
            };
            let den = w.re.mul(w.re)?.add(w.im.mul(w.im)?)?;
            steps.push(Step {
                description: format!(
                    "Multiply numerator and denominator by the conjugate {}.",
                    exact_plain(conj)
                ),
                description_latex: format!(
                    "Multiply numerator and denominator by the conjugate ${}$.",
                    exact_latex(conj)
                ),
            });
            steps.push(Step {
                description: format!("The denominator becomes |w|² = {}.", den.plain()),
                description_latex: format!("The denominator becomes $|w|^2 = {}$.", den.latex()),
            });
        }
        _ => {}
    }

    let mut argand = Argand::default();
    argand.points.push(point(
        "z",
        &format!("z = {}", exact_latex(z)),
        z.approx(),
        "input",
    ));
    argand.points.push(point(
        "w",
        &format!("w = {}", exact_latex(w)),
        w.approx(),
        "input",
    ));
    argand.points.push(point(
        label_plain,
        &format!("{label_latex} = {}", exact_latex(result)),
        result.approx(),
        "result",
    ));

    let mut lines = vec![
        format!("z = {}", exact_latex(z)),
        format!("w = {}", exact_latex(w)),
        format!("{label_latex} = {}", exact_latex(result)),
    ];
    let mut summary = vec![format!("{label_plain} = {}.", exact_plain(result))];
    let mut summary_latex = vec![format!("${label_latex} = {}$", exact_latex(result))];
    if !result.is_integral() {
        if let Some(decimal) = terminating_complex_decimal(result) {
            lines.push(format!("{label_latex} = {decimal}"));
            summary.push(format!("{label_plain} = {decimal}."));
            summary_latex.push(format!("${label_latex} = {decimal}$"));
        } else {
            lines.push(format!(
                "{label_latex} \\approx {}",
                approx_latex(result.approx())
            ));
            summary.push(format!(
                "{label_plain} ≈ {}.",
                approx_latex(result.approx())
            ));
            summary_latex.push(format!(
                "${label_latex} \\approx {}$",
                approx_latex(result.approx())
            ));
        }
    }

    Ok(Outcome {
        operation: "arithmetic",
        z,
        w: Some(w),
        steps,
        summary,
        summary_latex,
        lines,
        argand,
    })
}

pub fn polar_form(z: Exact) -> Outcome {
    let p = Polar::from(z);
    let za = z.approx();
    let mut argand = Argand::default();
    argand
        .points
        .push(point("z", &format!("z = {}", exact_latex(z)), za, "input"));
    if p.r > 0.00005 {
        argand.circles.push(Circle { radius: p.r });
    }

    let mod_plain = if p.mod_is_rational {
        format!("|z| = {}", p.mod_exact_plain)
    } else {
        format!("|z| = {} ≈ {}", p.mod_exact_plain, p.mod_approx)
    };
    let mod_latex = if p.mod_is_rational {
        format!("|z| = {}", p.mod_exact_latex)
    } else {
        format!("|z| = {} \\approx {}", p.mod_exact_latex, p.mod_approx)
    };
    let arg_plain = if p.arg_is_zero {
        "arg z = 0".to_string()
    } else {
        format!(
            "arg z = {} ≈ {} rad = {}°",
            p.arg_exact_plain, p.arg_approx, p.degrees
        )
    };
    let arg_latex = if p.arg_is_zero {
        "\\arg z = 0".to_string()
    } else {
        format!(
            "\\arg z = {} \\approx {}\\text{{ rad}} = {}^\\circ",
            p.arg_exact_latex, p.arg_approx, p.degrees
        )
    };

    let mut lines = vec![
        format!("z = {}", exact_latex(z)),
        mod_latex.clone(),
        arg_latex.clone(),
        format!("z = {}", p.form_exact_latex()),
        format!("z \\approx {}", p.form_approx_latex()),
        format!("z = {}", p.exp_exact_latex()),
    ];
    if p.r.abs() < 0.00005 {
        lines = vec!["z = 0".into(), "|z| = 0".into(), "z = 0".into()];
    }

    Outcome {
        operation: "polar",
        z,
        w: None,
        steps: vec![
            Step {
                description: format!("The modulus is {}.", mod_plain),
                description_latex: format!("The modulus is ${}$.", mod_latex),
            },
            Step {
                description: format!("The argument is {}.", arg_plain),
                description_latex: format!("The argument is ${}$.", arg_latex),
            },
        ],
        summary: vec![mod_plain, arg_plain],
        summary_latex: vec![format!("${}$", mod_latex), format!("${}$", arg_latex)],
        lines,
        argand,
    }
}

pub fn power(z: Exact, n: i64) -> Result<Outcome, String> {
    if !(0..=12).contains(&n) {
        return Err("use a power between 0 and 12".into());
    }
    let mut result = Exact::one();
    for _ in 0..n {
        result = result.mul(z)?;
    }
    let p = Polar::from(z);
    let mut argand = Argand::default();
    argand.points.push(point(
        "z",
        &format!("z = {}", exact_latex(z)),
        z.approx(),
        "input",
    ));
    argand.points.push(point(
        "z^n",
        &format!("z^{{{n}}} = {}", exact_latex(result)),
        result.approx(),
        "result",
    ));

    let nth_arg = if p.arg_is_zero {
        "0".to_string()
    } else if let Some((k, d)) = pi_fraction(p.theta * n as f64) {
        pi_fraction_latex(k, d)
    } else {
        format!("{n}\\left({}\\right)", p.arg_exact_latex)
    };
    let de_moivre = format!(
        "z^{{{n}}} = \\left({}\\right)^{{{n}}}\\left(\\cos {nth_arg} + i\\sin {nth_arg}\\right) = {}",
        p.mod_exact_latex,
        exact_latex(result)
    );
    let mut lines = vec![
        format!("z = {}", exact_latex(z)),
        format!("z = {}", p.form_exact_latex()),
        de_moivre,
    ];
    let mut summary = vec![format!("z^{{{n}}} = {}.", exact_plain(result))];
    let mut summary_latex = vec![format!("$z^{{{n}}} = {}$", exact_latex(result))];
    if !result.is_integral() {
        if let Some(decimal) = terminating_complex_decimal(result) {
            lines.push(format!("z^{{{n}}} = {decimal}"));
            summary.push(format!("z^{{{n}}} = {decimal}."));
            summary_latex.push(format!("$z^{{{n}}} = {decimal}$"));
        } else {
            lines.push(format!(
                "z^{{{n}}} \\approx {}",
                approx_latex(result.approx())
            ));
            summary.push(format!("z^{{{n}}} ≈ {}.", approx_latex(result.approx())));
            summary_latex.push(format!(
                "$z^{{{n}}} \\approx {}$",
                approx_latex(result.approx())
            ));
        }
    }

    Ok(Outcome {
        operation: "power",
        z,
        w: None,
        steps: vec![
            Step {
                description: format!(
                    "Write z in polar form: r = {} ≈ {} and θ = {} ≈ {} rad.",
                    p.mod_exact_plain, p.mod_approx, p.arg_exact_plain, p.arg_approx
                ),
                description_latex: format!(
                    "Write $z$ in polar form: $r = {} \\approx {}$ and $\\theta = {} \\approx {}\\text{{ rad}}$.",
                    p.mod_exact_latex, p.mod_approx, p.arg_exact_latex, p.arg_approx
                ),
            },
            Step {
                description: format!(
                    "De Moivre: z^{n} = r^{n}(cos nθ + i sin nθ). Here r^{n} = ({})^{n} and nθ = {}.",
                    p.mod_exact_plain, p.arg_exact_plain
                ),
                description_latex: format!(
                    "De Moivre: $z^{{{n}}} = r^{{{n}}}(\\cos {n}\\theta + i\\sin {n}\\theta)$. Here $r^{{{n}}} = \\left({}\\right)^{{{n}}}$ and $n\\theta = {nth_arg}$.",
                    p.mod_exact_latex
                ),
            },
        ],
        summary,
        summary_latex,
        lines,
        argand,
    })
}

fn omega_exact_complex(k: i64, n: i64) -> Option<Exact> {
    match n {
        1 => Some(Exact::one()),
        2 => {
            if k % 2 == 0 {
                Some(Exact::one())
            } else {
                Some(Exact {
                    re: Rat::int(-1),
                    im: Rat::ZERO,
                })
            }
        }
        4 => match k % 4 {
            0 => Some(Exact::one()),
            1 => Some(Exact {
                re: Rat::ZERO,
                im: Rat::ONE,
            }),
            2 => Some(Exact {
                re: Rat::int(-1),
                im: Rat::ZERO,
            }),
            _ => Some(Exact {
                re: Rat::ZERO,
                im: Rat::int(-1),
            }),
        },
        _ => None,
    }
}

fn omega_power_exact(k: i64, n: i64) -> String {
    let mut numerator = 2 * k;
    let mut denominator = n;
    let g = gcd(numerator, denominator);
    numerator /= g;
    denominator /= g;
    if denominator == 1 {
        if numerator == 1 {
            "e^{\\pi i}".to_string()
        } else {
            format!("e^{{{}\\pi i}}", numerator)
        }
    } else if numerator == 1 {
        format!("e^{{\\pi i/{denominator}}}")
    } else {
        format!("e^{{{}\\pi i/{denominator}}}", numerator)
    }
}

fn ordinal(n: i64) -> String {
    match n {
        1 => "1st".into(),
        2 => "2nd".into(),
        3 => "3rd".into(),
        _ => format!("{n}th"),
    }
}

fn nth_root_i64(value: i64, n: i64) -> Option<i64> {
    if value < 0 || n <= 0 {
        return None;
    }
    if value == 0 {
        return Some(0);
    }
    let root = (value as f64).powf(1.0 / n as f64).round() as i64;
    if root.pow(n as u32) == value {
        Some(root)
    } else {
        None
    }
}

fn nth_root_exact(value: Rat, n: i64) -> Option<Rat> {
    let num = nth_root_i64(value.n, n)?;
    let den = nth_root_i64(value.d, n)?;
    Rat::new(num as i128, den as i128).ok()
}

fn root_set_plain(n: i64, unity: bool) -> String {
    let z0 = if unity { "" } else { "z_0" };
    let term = |k: i64| {
        if unity {
            if k == 0 {
                "1".to_string()
            } else if k == 1 {
                "ω".to_string()
            } else {
                format!("ω^{k}")
            }
        } else if k == 0 {
            "z_0".to_string()
        } else if k == 1 {
            "z_0ω".to_string()
        } else {
            format!("z_0ω^{k}")
        }
    };
    match n {
        1 => format!("{{{}}}", if unity { "1" } else { z0 }),
        2 => format!("{{{}, {}}}", term(0), term(1)),
        3 => format!("{{{}, {}, {}}}", term(0), term(1), term(2)),
        _ => format!("{{{}, {}, …, {}}}", term(0), term(1), term(n - 1)),
    }
}

fn root_set_latex(n: i64, unity: bool) -> String {
    let term = |k: i64| {
        if unity {
            if k == 0 {
                "1".to_string()
            } else if k == 1 {
                "\\omega".to_string()
            } else {
                format!("\\omega^{{{k}}}")
            }
        } else if k == 0 {
            "z_0".to_string()
        } else if k == 1 {
            "z_0\\omega".to_string()
        } else {
            format!("z_0\\omega^{{{k}}}")
        }
    };
    match n {
        1 => format!("\\{{{}}}", term(0)),
        2 => format!("\\{{{}, {}\\}}", term(0), term(1)),
        3 => format!("\\{{{}, {}, {}\\}}", term(0), term(1), term(2)),
        _ => format!("\\{{{}, {}, \\ldots, {}\\}}", term(0), term(1), term(n - 1)),
    }
}

pub fn roots(z: Exact, n: i64) -> Result<Outcome, String> {
    if !(1..=12).contains(&n) {
        return Err("use a root index between 1 and 12".into());
    }
    let p = Polar::from(z);
    if p.r.abs() < 0.00005 {
        let mut argand = Argand::default();
        argand
            .points
            .push(point("1", "1", Approx { re: 0.0, im: 0.0 }, "root"));
        return Ok(Outcome {
            operation: "roots",
            z,
            w: None,
            steps: vec![Step {
                description: "The only nth root of 0 is 0.".into(),
                description_latex: "The only $n$th root of $0$ is $0$.".into(),
            }],
            summary: vec!["0 has one distinct nth root, 0.".into()],
            summary_latex: vec!["$0$ has one distinct $n$th root, $0$.".into()],
            lines: vec!["z = 0".into(), "0".into()],
            argand,
        });
    }

    let root_r = p.r.powf(1.0 / n as f64);
    let r2 =
        z.re.mul(z.re)
            .and_then(|a| z.im.mul(z.im).and_then(|b| a.add(b)))
            .unwrap_or(Rat::ZERO);
    let root_mod_rat = sqrt_exact(r2).and_then(|r| nth_root_exact(r, n));
    let root_mod_exact = match root_mod_rat {
        Some(r) => r.latex(),
        None => {
            if p.mod_exact_latex.contains('\\') {
                format!("\\left({}\\right)^{{1/{n}}}", p.mod_exact_latex)
            } else {
                format!("{}^{{1/{n}}}", p.mod_exact_latex)
            }
        }
    };
    let z0_approx = Approx {
        re: root_r * (p.theta / n as f64).cos(),
        im: root_r * (p.theta / n as f64).sin(),
    };
    let principal_angle = if p.arg_is_zero {
        "0".to_string()
    } else if let Some((k, d)) = pi_fraction_up_to(p.theta / n as f64, 12 * n) {
        pi_fraction_latex(k, d)
    } else {
        format!("\\frac{{{}}}{{{n}}}", p.arg_exact_latex)
    };
    let z0_exact = if p.arg_is_zero {
        root_mod_exact.clone()
    } else {
        format!("{}e^{{i{principal_angle}}}", root_mod_exact)
    };
    let z0_needs_approx = !(p.arg_is_zero && root_mod_rat.is_some());
    let is_unity = p.arg_is_zero && (p.r - 1.0).abs() < 1e-12;

    let mod_line = if p.mod_is_rational {
        format!("|z| = {}", p.mod_exact_latex)
    } else {
        format!("|z| = {} \\approx {}", p.mod_exact_latex, p.mod_approx)
    };
    let arg_line = if p.arg_is_zero {
        "\\arg z = 0".to_string()
    } else {
        format!(
            "\\arg z = {} \\approx {}\\text{{ rad}}",
            p.arg_exact_latex, p.arg_approx
        )
    };
    let omega_angle = 2.0 * PI / n as f64;
    let root_values: Vec<Approx> = (0..n)
        .map(|k| {
            let angle = (p.theta + 2.0 * PI * k as f64) / n as f64;
            Approx {
                re: root_r * angle.cos(),
                im: root_r * angle.sin(),
            }
        })
        .collect();

    let mut lines = vec![format!("z = {}", exact_latex(z)), mod_line, arg_line];
    let mut argand = Argand::default();

    if is_unity {
        lines.push("1".to_string());
        argand.points.push(point("1", "1", root_values[0], "root"));
        for k in 1..n {
            let omega_label = if k == 1 {
                "\\omega".to_string()
            } else {
                format!("\\omega^{{{k}}}")
            };
            let exact = omega_power_exact(k, n);
            if let Some(value) = omega_exact_complex(k, n) {
                lines.push(format!("{omega_label} = {exact} = {}", exact_latex(value)));
            } else {
                let omega_value = Approx {
                    re: (omega_angle * k as f64).cos(),
                    im: (omega_angle * k as f64).sin(),
                };
                lines.push(format!(
                    "{omega_label} = {exact} \\approx {}",
                    approx_latex(omega_value)
                ));
            }
            argand.points.push(point(
                &omega_label,
                &format!("{omega_label} = {exact}"),
                root_values[k as usize],
                "root",
            ));
        }
    } else {
        lines.push(if z0_needs_approx {
            format!("z_0 = {} \\approx {}", z0_exact, approx_latex(z0_approx))
        } else {
            format!("z_0 = {}", z0_exact)
        });
        argand.points.push(point(
            "z",
            &format!("z = {}", exact_latex(z)),
            z.approx(),
            "input",
        ));
        argand.points.push(point(
            "z_0",
            &format!("z_0 = {}", z0_exact),
            root_values[0],
            "root",
        ));
        for k in 1..n {
            let omega_label = if k == 1 {
                "\\omega".to_string()
            } else {
                format!("\\omega^{{{k}}}")
            };
            let exact = omega_power_exact(k, n);
            if let Some(value) = omega_exact_complex(k, n) {
                lines.push(format!("{omega_label} = {exact} = {}", exact_latex(value)));
            } else {
                let omega_value = Approx {
                    re: (omega_angle * k as f64).cos(),
                    im: (omega_angle * k as f64).sin(),
                };
                lines.push(format!(
                    "{omega_label} = {exact} \\approx {}",
                    approx_latex(omega_value)
                ));
            }
        }
        for k in 1..n {
            let power = if k == 1 {
                "\\omega".to_string()
            } else {
                format!("\\omega^{{{k}}}")
            };
            let exact_root =
                if let (Some(z0), Some(omega)) = (root_mod_rat, omega_exact_complex(k, n)) {
                    Some(
                        Exact {
                            re: z0,
                            im: Rat::ZERO,
                        }
                        .mul(omega)?,
                    )
                } else {
                    None
                };
            if let Some(root) = exact_root {
                lines.push(format!("z_0{power} = {}", exact_latex(root)));
                argand.points.push(point(
                    &format!("z_0{power}"),
                    &format!("z_0{power} = {}", exact_latex(root)),
                    root_values[k as usize],
                    "root",
                ));
            } else {
                let exact = omega_power_exact(k, n);
                lines.push(format!(
                    "z_0{power} \\approx {}",
                    approx_latex(root_values[k as usize])
                ));
                argand.points.push(point(
                    &format!("z_0{power}"),
                    &format!("z_0{power} = z_0 {exact}"),
                    root_values[k as usize],
                    "root",
                ));
            }
        }
    }
    argand.circles.push(Circle { radius: root_r });

    let summary_plain = format!(
        "The roots are {}, with ω = e^(2πi/{n}).",
        root_set_plain(n, is_unity)
    );
    let summary_latex = format!(
        "The roots are ${}$, with $\\omega = e^{{2\\pi i/{n}}}$.",
        root_set_latex(n, is_unity)
    );

    let mut steps = vec![Step {
        description: format!(
            "The modulus is {} and the argument is {}.",
            if p.mod_is_rational {
                format!("|z| = {}", p.mod_exact_plain)
            } else {
                format!("|z| = {} ≈ {}", p.mod_exact_plain, p.mod_approx)
            },
            if p.arg_is_zero {
                "arg z = 0".to_string()
            } else {
                format!("arg z = {} ≈ {} rad", p.arg_exact_plain, p.arg_approx)
            }
        ),
        description_latex: format!(
            "The modulus is ${}$ and the argument is ${}$.",
            if p.mod_is_rational {
                format!("|z| = {}", p.mod_exact_latex)
            } else {
                format!("|z| = {} \\approx {}", p.mod_exact_latex, p.mod_approx)
            },
            if p.arg_is_zero {
                "\\arg z = 0".to_string()
            } else {
                format!(
                    "\\arg z = {} \\approx {}\\text{{ rad}}",
                    p.arg_exact_latex, p.arg_approx
                )
            }
        ),
    }];
    if !is_unity {
        steps.push(Step {
            description: format!(
                "A principal {} root is z_0 = |z|^(1/{n}) e^(iθ/{n}).",
                ordinal(n)
            ),
            description_latex: format!(
                "A principal {} root is $z_0 = |z|^{{1/{n}}}e^{{i\\theta/{n}}}$.",
                ordinal(n)
            ),
        });
    }
    steps.push(Step {
        description: format!(
            "Let ω = e^(2πi/{n}). The roots are {}.",
            root_set_plain(n, is_unity)
        ),
        description_latex: format!(
            "Let $\\omega = e^{{2\\pi i/{n}}}$. The roots are ${}$.",
            root_set_latex(n, is_unity)
        ),
    });
    Ok(Outcome {
        operation: "roots",
        z,
        w: None,
        steps,
        summary: vec![summary_plain],
        summary_latex: vec![summary_latex],
        lines,
        argand,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn z(a: i64, b: i64) -> Exact {
        Exact::new(Rat::int(a), Rat::int(b))
    }

    #[test]
    fn rounds_then_removes_trailing_zeroes() {
        assert_eq!(fmt4(1.23456), "1.2346");
        assert_eq!(fmt4(-1.23456), "-1.2346");
        assert_eq!(fmt4(1.99999), "2");
        assert_eq!(fmt4(0.866025), "0.866");
        assert_eq!(fmt4(-0.5), "-0.5");
        assert_eq!(fmt4(0.00004), "0");
        assert_eq!(fmt4(-0.00004), "0");
        assert_eq!(fmt4(0.00009), "0.0001");
    }

    #[test]
    fn multiplies_and_divides_exactly_with_approximations() {
        let out = arithmetic(z(3, 4), z(1, -2), "*").unwrap();
        assert!(out.lines[2].contains("11 - 2i"), "{:?}", out.lines);
        assert!(!out.lines.iter().any(|l| l.contains("approx")));
        let out = arithmetic(z(1, 2), z(3, -4), "/").unwrap();
        assert!(
            out.lines.iter().any(|l| l.contains("\\frac{z}{w}")),
            "{:?}",
            out.lines
        );
        assert!(
            out.lines.iter().any(|l| l.contains("0.4i")),
            "{:?}",
            out.lines
        );
    }

    #[test]
    fn repeating_decimals_use_approx_and_terminating_ones_use_equals() {
        let terminating = arithmetic(z(1, 2), z(3, -4), "/").unwrap();
        assert!(
            terminating
                .lines
                .iter()
                .any(|l| l.contains("= -0.2 + 0.4i")),
            "{:?}",
            terminating.lines
        );
        let repeating = arithmetic(z(1, 0), z(3, 0), "/").unwrap();
        assert!(
            repeating
                .lines
                .iter()
                .any(|l| l.contains("\\approx 0.3333")),
            "{:?}",
            repeating.lines
        );
        assert!(
            !repeating.lines.iter().any(|l| l.contains("= 0.3333")),
            "{:?}",
            repeating.lines
        );
    }

    #[test]
    fn polar_form_has_exact_and_approximate_values() {
        let out = polar_form(z(1, 1));
        assert!(
            out.lines.iter().any(|l| l.contains("\\sqrt{2}")),
            "{:?}",
            out.lines
        );
        assert!(
            out.lines.iter().any(|l| l.contains("1.4142")),
            "{:?}",
            out.lines
        );
        assert!(
            out.lines.iter().any(|l| l.contains("\\frac{\\pi}{4}")),
            "{:?}",
            out.lines
        );
        assert!(
            out.lines.iter().any(|l| l.contains("0.7854")),
            "{:?}",
            out.lines
        );
    }

    #[test]
    fn power_is_labelled_with_the_exponent() {
        let out = power(z(1, 1), 6).unwrap();
        assert!(
            out.lines
                .iter()
                .any(|l| l.contains("z^{6}") && l.contains("-8i")),
            "{:?}",
            out.lines
        );
        assert!(out.summary[0].contains("z^{6}"), "{:?}", out.summary);
    }

    #[test]
    fn roots_have_exact_and_approximate_forms() {
        let out = roots(z(1, 0), 3).unwrap();
        assert_eq!(out.argand.points.len(), 3);
        assert!(
            out.lines.iter().any(|l| l.contains("2\\pi")),
            "{:?}",
            out.lines
        );
        assert!(
            out.lines.iter().any(|l| l.contains("0.866")),
            "{:?}",
            out.lines
        );
        let zero = roots(z(0, 0), 4).unwrap();
        assert_eq!(zero.argand.points.len(), 1);
    }
}
