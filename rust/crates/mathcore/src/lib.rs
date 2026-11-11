//! Exact rational arithmetic and the symbolic `a + bM` value used by Big-M.
//!
//! Deliberately small: no big integers, no floating point. A-level and early
//! undergraduate problems stay well inside `i64` once reduced; overflowing
//! returns an error rather than a silently wrong answer.

use std::cmp::Ordering;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rat {
    pub n: i64,
    pub d: i64,
}

impl Rat {
    pub const ZERO: Rat = Rat { n: 0, d: 1 };
    pub const ONE: Rat = Rat { n: 1, d: 1 };

    pub fn new(n: i128, d: i128) -> Result<Self, String> {
        if d == 0 {
            return Err("division by zero".into());
        }
        let (n, d) = if d < 0 { (-n, -d) } else { (n, d) };
        let g = gcd(n, d);
        let (n, d) = if g == 0 { (n, d) } else { (n / g, d / g) };
        if n < i64::MIN as i128 || n > i64::MAX as i128 || d > i64::MAX as i128 {
            return Err("number too large".into());
        }
        Ok(Rat {
            n: n as i64,
            d: d as i64,
        })
    }

    pub fn int(n: i64) -> Self {
        Rat { n, d: 1 }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if s.is_empty() {
            return Err("enter a number".into());
        }
        if s.len() > 40 {
            return Err("number too long".into());
        }
        if let Some((a, b)) = s.split_once('/') {
            let n = parse_i128(a)?;
            let d = parse_i128(b)?;
            return Rat::new(n, d);
        }
        if let Some((int_part, frac_part)) = s.split_once('.') {
            let frac_part = frac_part.trim();
            if frac_part.len() > 9 {
                return Err("too many decimal places".into());
            }
            let sign = if int_part.trim_start().starts_with('-') {
                -1i128
            } else {
                1i128
            };
            let int_digits = int_part.trim_start_matches(['+', '-']);
            let digits = format!(
                "{}{}",
                if int_digits.is_empty() { "0" } else { int_digits },
                frac_part
            );
            let n: i128 = digits
                .parse()
                .map_err(|_| format!("invalid number: {s}"))?;
            let d = 10i128.pow(frac_part.len() as u32);
            return Rat::new(sign * n, d);
        }
        Rat::new(parse_i128(s)?, 1)
    }

    pub fn add(self, o: Self) -> Result<Self, String> {
        Rat::new(
            self.n as i128 * o.d as i128 + o.n as i128 * self.d as i128,
            self.d as i128 * o.d as i128,
        )
    }

    pub fn sub(self, o: Self) -> Result<Self, String> {
        Rat::new(
            self.n as i128 * o.d as i128 - o.n as i128 * self.d as i128,
            self.d as i128 * o.d as i128,
        )
    }

    pub fn mul(self, o: Self) -> Result<Self, String> {
        Rat::new(
            self.n as i128 * o.n as i128,
            self.d as i128 * o.d as i128,
        )
    }

    pub fn div(self, o: Self) -> Result<Self, String> {
        if o.n == 0 {
            return Err("division by zero".into());
        }
        Rat::new(
            self.n as i128 * o.d as i128,
            self.d as i128 * o.n as i128,
        )
    }

    pub fn neg(self) -> Self {
        Rat {
            n: -self.n,
            d: self.d,
        }
    }

    pub fn abs(self) -> Self {
        Rat {
            n: if self.n < 0 {
                self.n.checked_neg().unwrap_or(self.n)
            } else {
                self.n
            },
            d: self.d,
        }
    }

    pub fn is_zero(self) -> bool {
        self.n == 0
    }

    pub fn is_negative(self) -> bool {
        self.n < 0
    }

    pub fn is_positive(self) -> bool {
        self.n > 0
    }

    pub fn cmp_rat(self, o: Self) -> Ordering {
        (self.n as i128 * o.d as i128).cmp(&(o.n as i128 * self.d as i128))
    }

    pub fn plain(self) -> String {
        if self.d == 1 {
            self.n.to_string()
        } else if self.n < 0 {
            format!("-{}/{}", self.n.unsigned_abs(), self.d)
        } else {
            format!("{}/{}", self.n, self.d)
        }
    }

    pub fn latex(self) -> String {
        if self.d == 1 {
            self.n.to_string()
        } else if self.n < 0 {
            format!("-\\frac{{{}}}{{{}}}", self.n.unsigned_abs(), self.d)
        } else {
            format!("\\frac{{{}}}{{{}}}", self.n, self.d)
        }
    }
}

impl PartialOrd for Rat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp_rat(*other))
    }
}

impl Ord for Rat {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_rat(*other)
    }
}

impl fmt::Display for Rat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.plain())
    }
}

/// `a + bM`, interpreted with `M` tending to infinity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MVal {
    pub a: Rat,
    pub b: Rat,
}

impl MVal {
    pub const ZERO: MVal = MVal {
        a: Rat::ZERO,
        b: Rat::ZERO,
    };
    pub const ONE: MVal = MVal {
        a: Rat::ONE,
        b: Rat::ZERO,
    };

    pub fn rat(a: Rat) -> Self {
        MVal { a, b: Rat::ZERO }
    }

    pub fn add(self, o: Self) -> Result<Self, String> {
        Ok(MVal {
            a: self.a.add(o.a)?,
            b: self.b.add(o.b)?,
        })
    }

    pub fn sub(self, o: Self) -> Result<Self, String> {
        Ok(MVal {
            a: self.a.sub(o.a)?,
            b: self.b.sub(o.b)?,
        })
    }

    pub fn mul_rat(self, r: Rat) -> Result<Self, String> {
        Ok(MVal {
            a: self.a.mul(r)?,
            b: self.b.mul(r)?,
        })
    }

    pub fn div_rat(self, r: Rat) -> Result<Self, String> {
        Ok(MVal {
            a: self.a.div(r)?,
            b: self.b.div(r)?,
        })
    }

    pub fn neg(self) -> Self {
        MVal {
            a: self.a.neg(),
            b: self.b.neg(),
        }
    }

    pub fn is_zero(self) -> bool {
        self.a.is_zero() && self.b.is_zero()
    }

    pub fn is_negative(self) -> bool {
        self.b.is_negative() || (self.b.is_zero() && self.a.is_negative())
    }

    pub fn is_positive(self) -> bool {
        self.b.is_positive() || (self.b.is_zero() && self.a.is_positive())
    }

    /// Ordering for `M -> infinity`: compare the coefficient of M first.
    pub fn cmp_m(self, o: Self) -> Ordering {
        match self.b.cmp_rat(o.b) {
            Ordering::Equal => self.a.cmp_rat(o.a),
            other => other,
        }
    }

    pub fn plain(self) -> String {
        if self.b.is_zero() {
            return self.a.plain();
        }
        let m = if self.b == Rat::ONE {
            "M".to_string()
        } else if self.b == Rat::int(-1) {
            "-M".to_string()
        } else {
            format!("{}M", self.b.plain())
        };
        if self.a.is_zero() {
            m
        } else if self.b.is_negative() {
            if self.b == Rat::int(-1) {
                format!("{} - M", self.a.plain())
            } else {
                format!("{} - {}M", self.a.plain(), self.b.abs().plain())
            }
        } else {
            format!("{} + {}", self.a.plain(), m)
        }
    }

    /// Magnitude, with the M term first when that avoids a leading negative.
    pub fn abs_plain(self) -> String {
        let v = if self.is_negative() { self.neg() } else { self };
        if v.b.is_zero() {
            return v.a.plain();
        }
        let m = if v.b == Rat::ONE {
            "M".to_string()
        } else {
            format!("{}M", v.b.plain())
        };
        if v.a.is_zero() {
            m
        } else if v.a.is_positive() {
            format!("{} + {}", v.a.plain(), m)
        } else {
            format!("{} - {}", m, v.a.abs().plain())
        }
    }

    pub fn abs_latex(self) -> String {
        let v = if self.is_negative() { self.neg() } else { self };
        if v.b.is_zero() {
            return v.a.latex();
        }
        let m = if v.b == Rat::ONE {
            "M".to_string()
        } else {
            format!("{}M", v.b.latex())
        };
        if v.a.is_zero() {
            m
        } else if v.a.is_positive() {
            format!("{} + {}", v.a.latex(), m)
        } else {
            format!("{} - {}", m, v.a.abs().latex())
        }
    }

    pub fn latex(self) -> String {
        if self.b.is_zero() {
            return self.a.latex();
        }
        let m = if self.b == Rat::ONE {
            "M".to_string()
        } else if self.b == Rat::int(-1) {
            "-M".to_string()
        } else {
            format!("{}M", self.b.latex())
        };
        if self.a.is_zero() {
            m
        } else if self.b.is_negative() {
            if self.b == Rat::int(-1) {
                format!("{} - M", self.a.latex())
            } else {
                format!("{} - {}M", self.a.latex(), self.b.abs().latex())
            }
        } else {
            format!("{} + {}", self.a.latex(), m)
        }
    }
}

fn gcd(mut a: i128, mut b: i128) -> i128 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    if a == 0 {
        0
    } else {
        a
    }
}

fn parse_i128(s: &str) -> Result<i128, String> {
    let t = s.trim();
    if t.is_empty() {
        return Err("enter a number".into());
    }
    t.parse::<i128>()
        .map_err(|_| format!("invalid number: {}", s.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_integers_decimals_and_fractions() {
        assert_eq!(Rat::parse("3").unwrap(), Rat::int(3));
        assert_eq!(Rat::parse("-0.25").unwrap(), Rat::new(-1, 4).unwrap());
        assert_eq!(Rat::parse("3/6").unwrap(), Rat::new(1, 2).unwrap());
        assert_eq!(Rat::parse(".5").unwrap(), Rat::new(1, 2).unwrap());
    }

    #[test]
    fn reduces_and_formats() {
        let r = Rat::new(6, -8).unwrap();
        assert_eq!(r, Rat::new(-3, 4).unwrap());
        assert_eq!(r.plain(), "-3/4");
        assert_eq!(r.latex(), "-\\frac{3}{4}");
    }

    #[test]
    fn m_ordering_is_lexicographic_in_b_then_a() {
        let small = MVal {
            a: Rat::int(100),
            b: Rat::int(1),
        };
        let big = MVal {
            a: Rat::int(-100),
            b: Rat::int(2),
        };
        assert_eq!(small.cmp_m(big), Ordering::Less);
    }

    #[test]
    fn m_formatting() {
        assert_eq!(
            MVal {
                a: Rat::int(3),
                b: Rat::int(-2)
            }
            .latex(),
            "3 - 2M"
        );
        assert_eq!(
            MVal {
                a: Rat::ZERO,
                b: Rat::int(-1)
            }
            .plain(),
            "-M"
        );
    }
}
