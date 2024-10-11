use crate::{Duration, Speed};
use std::ops::{Add, Div, DivAssign, Mul, MulAssign};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    pub const fn millimeter(&self) -> u64 {
        self.millimeter
    }

    #[must_use]
    pub fn meter_f32(&self) -> f32 {
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
