use crate::{Duration, Error, Speed};
use serde::Deserialize;
use std::ops::{Add, Div, DivAssign, Mul, MulAssign};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(try_from = "String")]
pub struct Distance {
    millimeter: u64,
}

impl Distance {
    pub const ZERO: Self = Self::from_millimeter(0);
    pub const ADJACENT: Self = Self::from_millimeter(1000);
    pub const DIAGONAL: Self = Self::from_millimeter(1414);
    pub const VERTICAL: Self = Self::from_millimeter(1800);

    #[must_use]
    pub const fn from_millimeter(millimeter: u64) -> Self {
        Self { millimeter }
    }

    #[must_use]
    pub const fn millimeter(self) -> u64 {
        self.millimeter
    }

    #[must_use]
    pub const fn meter_f32(self) -> f32 {
        0.001 * self.millimeter as f32
    }
}

impl Add<Self> for Distance {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::from_millimeter(self.millimeter + other.millimeter)
    }
}

impl Div<u64> for Distance {
    type Output = Self;

    fn div(self, div: u64) -> Self {
        Self::from_millimeter(self.millimeter / div)
    }
}

impl Div<Speed> for Distance {
    type Output = Duration;

    fn div(self, speed: Speed) -> Duration {
        Duration::SECOND * self.millimeter / speed.millimeter_per_second()
    }
}

impl DivAssign<u64> for Distance {
    fn div_assign(&mut self, div: u64) {
        self.millimeter /= div;
    }
}

impl Mul<u64> for Distance {
    type Output = Self;

    fn mul(self, factor: u64) -> Self {
        Self::from_millimeter(self.millimeter * factor)
    }
}

impl MulAssign<u64> for Distance {
    fn mul_assign(&mut self, factor: u64) {
        self.millimeter *= factor;
    }
}

impl TryFrom<&str> for Distance {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Error> {
        let quantity = value.trim_matches(char::is_alphabetic).trim();
        let unit: String = value.matches(char::is_alphabetic).collect();
        //trace!("{value} {} {}", &quantity, &unit);

        let quantity = quantity.parse::<f32>()?;

        Ok(Self {
            millimeter: match unit.to_lowercase().as_str() {
                "km" => 1_000_000.0 * quantity,
                "m" | "meter" => 1_000.0 * quantity,
                "cm" => 10.0 * quantity,
                "mm" => quantity,
                _ => {
                    return Err(Error::UnknowUnit {
                        _value: String::from(value),
                    });
                }
            } as u64,
        })
    }
}

impl TryFrom<String> for Distance {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Error> {
        Self::try_from(value.as_ref())
    }
}

#[cfg(test)]
mod volume_tests {
    use super::*;

    #[test]
    fn parsing_works() {
        assert_eq!(Distance::try_from("21 mm"), Ok(Distance { millimeter: 21 }));
        assert_eq!(
            Distance::try_from("78.9 cM"),
            Ok(Distance { millimeter: 789 })
        );
        assert_eq!(
            Distance::try_from("35.6M"),
            Ok(Distance { millimeter: 35_600 })
        );
    }
}
