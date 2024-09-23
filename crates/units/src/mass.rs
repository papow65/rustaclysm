use serde::Deserialize;
use std::{
    fmt,
    ops::{Add, Div, Sub},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Deserialize)]
#[serde(from = "String")]
pub struct Mass {
    milligram: u64,
}

impl Mass {
    pub const ZERO: Self = Self { milligram: 0 };
}

impl Add<Self> for Mass {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            milligram: self.milligram + other.milligram,
        }
    }
}

impl Sub<Self> for Mass {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            milligram: self.milligram - other.milligram,
        }
    }
}

impl Div<Self> for Mass {
    type Output = u32;
    fn div(self, other: Self) -> Self::Output {
        (self.milligram / other.milligram) as Self::Output
    }
}

impl fmt::Display for Mass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if 10_000_000 <= self.milligram {
            write!(f, "{} kg", self.milligram / 1_000_000)
        } else if 1_000_000 <= self.milligram {
            write!(f, "{:.1} kg", self.milligram as f32 * 0.000_001)
        } else if 1_000 <= self.milligram {
            write!(f, "{} g", self.milligram / 1000)
        } else {
            write!(f, "{} mg", self.milligram)
        }
    }
}

impl<S: AsRef<str>> From<S> for Mass {
    fn from(value: S) -> Self {
        let value = value.as_ref();
        let quantity = value.trim_matches(char::is_alphabetic).trim();
        let unit: String = value.matches(char::is_alphabetic).collect();
        //println!("{value} {} {}", &quantity, &unit);

        let quantity = quantity
            .parse::<f32>()
            .unwrap_or_else(|err| panic!("{err:?} when parsing {quantity:?}"));

        Self {
            milligram: match unit.to_lowercase().as_str() {
                "mg" => quantity,
                "g" => 1_000.0 * quantity,
                "kg" => 1_000_000.0 * quantity,
                _ => panic!("{value} {quantity} {}", &unit),
            } as u64,
        }
    }
}

#[cfg(test)]
mod mass_tests {
    use super::*;

    #[test]
    fn parsing_works() {
        assert_eq!(Mass::from(String::from("21mg")), Mass { milligram: 21 });
        assert_eq!(
            Mass::from(String::from("35.6 Kg")),
            Mass {
                milligram: 35_600_000
            }
        );
    }
}
