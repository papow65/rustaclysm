use serde::Deserialize;
use std::{
    fmt,
    iter::Sum,
    ops::{Add, Div, Sub},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Deserialize)]
#[serde(from = "String")]
pub struct Volume {
    milliliter: u64,
}

impl Volume {
    pub const ZERO: Self = Self { milliliter: 0 };
}

impl Add<Self> for Volume {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            milliliter: self.milliliter + other.milliliter,
        }
    }
}

impl Sub<Self> for Volume {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            milliliter: self.milliliter - other.milliliter,
        }
    }
}

impl Div<Self> for Volume {
    type Output = u32;
    fn div(self, other: Self) -> Self::Output {
        (self.milliliter / other.milliliter) as Self::Output
    }
}

impl fmt::Display for Volume {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if 10_000 <= self.milliliter {
            write!(f, "{} l", self.milliliter / 1000)
        } else if 1_000 <= self.milliliter {
            write!(f, "{:.1} l", self.milliliter as f32 * 0.001)
        } else {
            write!(f, "{} ml", self.milliliter)
        }
    }
}

impl<S: AsRef<str>> From<S> for Volume {
    fn from(value: S) -> Self {
        let value = value.as_ref();
        let quantity = value.trim_matches(char::is_alphabetic).trim();
        let unit: String = value.matches(char::is_alphabetic).collect();
        //trace!("{value} {} {}", &quantity, &unit);

        let quantity = quantity
            .parse::<f32>()
            .unwrap_or_else(|err| panic!("{err:?} when parsing {quantity:?}"));

        Self {
            milliliter: match unit.to_lowercase().as_str() {
                "l" => 1_000.0 * quantity,
                "ml" => quantity,
                _ => panic!("{value} {quantity} {}", &unit),
            } as u64,
        }
    }
}

impl Sum for Volume {
    fn sum<V>(iter: V) -> Self
    where
        V: Iterator<Item = Self>,
    {
        iter.fold(Self { milliliter: 0 }, |a, b| Self {
            milliliter: a.milliliter + b.milliliter,
        })
    }
}

#[cfg(test)]
mod volume_tests {
    use super::*;

    #[test]
    fn parsing_works() {
        assert_eq!(Volume::from("21 ml"), Volume { milliliter: 21 });
        assert_eq!(Volume::from("35.6L"), Volume { milliliter: 35_600 });
    }
}
