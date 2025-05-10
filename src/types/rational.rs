//! Rational number type that uses `heapless::String` for all fallible APIs
//! so it works in `#![no_std]` / allocator‑free environments.

use core::cmp::Ordering;
use core::convert::TryFrom;
use core::fmt;
use core::ops::Sub;
use core::str::FromStr;

use heapless::String as HString;

/// Convenience alias for error strings used in this module.
/// 64 bytes is plenty for the hard‑coded messages below.
/// Adjust upward if you add longer messages later.
pub type ErrorString = HString<64>;

#[inline]
pub fn err(msg: &'static str) -> ErrorString {
    let mut s = ErrorString::new();
    s.push_str(msg).unwrap(); // we know all literals fit
    s
}

/// Greatest‑common‑divisor (Euclid).
fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a.abs()
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Rational {
    num: i64,
    den: i64,
}

impl Rational {
    /// Construct a new reduced fraction. Sign is always on the numerator.
    pub fn new(num: i64, den: i64) -> Result<Self, ErrorString> {
        if den == 0 {
            return Err(err("Denominator cannot be zero"));
        }
        let sign = if den < 0 { -1 } else { 1 };
        let g = gcd(num, den);
        Ok(Rational {
            num: sign * (num / g),
            den: den.abs() / g,
        })
    }

    /// Parse from a decimal string such as "-12.34" or "7".
    pub fn from_decimal_str(s: &str) -> Result<Self, ErrorString> {
        let s = s.trim();
        if let Some(dot) = s.find('.') {
            let (int_part, frac_part) = s.split_at(dot);
            let frac = &frac_part[1..];
            let base: i64 = 10i64.pow(frac.len() as u32);
            let int_val: i64 = int_part.parse().map_err(|_| err("Invalid integer part"))?;
            let frac_val: i64 = frac.parse().map_err(|_| err("Invalid fractional part"))?;
            let signed = if int_val < 0 { -1 } else { 1 };
            let numerator = int_val.abs() * base + frac_val;
            Rational::new(signed * numerator, base)
        } else {
            let i: i64 = s.parse().map_err(|_| err("Invalid integer"))?;
            Rational::new(i, 1)
        }
    }

    /// Checked subtraction that propagates overflow / zero‑denominator issues.
    pub fn checked_sub(&self, rhs: &Rational) -> Result<Rational, ErrorString> {
        let num = self
            .num
            .checked_mul(rhs.den)
            .and_then(|ad| rhs.num.checked_mul(self.den).map(|cb| ad - cb))
            .ok_or_else(|| err("Overflow computing numerator"))?;
        let den = self
            .den
            .checked_mul(rhs.den)
            .ok_or_else(|| err("Overflow computing denominator"))?;
        Rational::new(num, den)
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            return write!(f, "{}", self.num);
        }
        let value = self.num as f64 / self.den as f64;
        write!(f, "{}", value)
    }
}

impl fmt::Debug for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl FromStr for Rational {
    type Err = ErrorString;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Rational::from_decimal_str(s)
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (self.num.checked_mul(other.den)?).partial_cmp(&(other.num.checked_mul(self.den)?))
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .expect("overflow in Rational comparison")
    }
}

impl Sub for Rational {
    type Output = Rational;

    fn sub(self, rhs: Rational) -> Rational {
        self.checked_sub(&rhs)
            .expect("Rational subtraction failed: overflow or zero denominator")
    }
}
