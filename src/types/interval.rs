//! Interval<T> with heapless error strings so it works without `std`.

use crate::types::rational::{err, ErrorString, Rational};
use core::cmp::Ordering;
use core::fmt::Debug;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interval<T: Ord> {
    pub min: T,
    pub max: T,
}

impl<T: Ord> Interval<T> {
    pub fn new(min: T, max: T) -> Result<Self, ErrorString> {
        if min <= max {
            Ok(Self { min, max })
        } else {
            Err(err("min must be ≤ max"))
        }
    }
}

impl<T: Ord> PartialOrd for Interval<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.min
                .cmp(&other.min)
                .then_with(|| self.max.cmp(&other.max)),
        )
    }
}

impl<T: Ord> Ord for Interval<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Interval<Rational> {
    pub fn from_strs(min_s: &str, max_s: &str) -> Result<Self, ErrorString> {
        let min = Rational::from_decimal_str(min_s)?;
        let max = Rational::from_decimal_str(max_s)?;
        Interval::new(min, max)
    }
}

impl Interval<u128> {
    pub fn from_ints(min: u128, max: u128) -> Result<Self, ErrorString> {
        Interval::new(min, max)
    }

    pub fn from_strs(min_s: &str, max_s: &str) -> Result<Self, ErrorString> {
        let min = min_s
            .parse::<u128>()
            .map_err(|_| err("invalid min integer"))?;
        let max = max_s
            .parse::<u128>()
            .map_err(|_| err("invalid max integer"))?;
        Interval::new(min, max)
    }
}
