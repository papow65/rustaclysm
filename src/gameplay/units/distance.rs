use super::{Duration, Speed};
use std::ops::{Add, Div, DivAssign, Mul, MulAssign};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Distance {
    millimeter: u64,
}

impl Distance {
    pub(crate) const ZERO: Self = Self::from_millimeter(0);
    pub(crate) const ADJACENT: Self = Self::from_millimeter(1000);
    pub(crate) const DIAGONAL: Self = Self::from_millimeter(1414);
    pub(crate) const VERTICAL: Self = Self::from_millimeter(1800);

    pub(crate) const fn from_millimeter(millimeter: u64) -> Self {
        Self { millimeter }
    }

    pub(crate) const fn millimeter(&self) -> u64 {
        self.millimeter
    }

    pub(crate) fn meter_f32(&self) -> f32 {
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
        Duration(self.millimeter * 1000 / speed.millimeter_per_second())
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
