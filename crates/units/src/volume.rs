use serde::Deserialize;
use std::{
    fmt,
    iter::Sum,
    ops::{Add, Div, Sub},
};

use crate::Error;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Deserialize)]
#[serde(try_from = "String")]
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

impl TryFrom<&str> for Volume {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Error> {
        let quantity = value.trim_matches(char::is_alphabetic).trim();
        let unit: String = value.matches(char::is_alphabetic).collect();
        //trace!("{value} {} {}", &quantity, &unit);

        let quantity = quantity.parse::<f32>()?;

        Ok(Self {
            milliliter: match unit.to_lowercase().as_str() {
                "l" => 1_000.0 * quantity,
                "ml" => quantity,
                _ => {
                    return Err(Error::UnknowUnit {
                        _value: String::from(value),
                    });
                }
            } as u64,
        })
    }
}

impl TryFrom<String> for Volume {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Error> {
        Self::try_from(value.as_ref())
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
        assert_eq!(Volume::try_from("21 ml"), Ok(Volume { milliliter: 21 }));
        assert_eq!(Volume::try_from("35.6L"), Ok(Volume { milliliter: 35_600 }));
    }
}
