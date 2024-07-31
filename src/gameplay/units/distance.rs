use super::{Duration, Speed};
use std::ops::{Add, Div};

#[derive(Clone, Copy, Debug)]
pub(crate) struct Distance(pub(crate) u64);

impl Distance {
    pub(crate) const ZERO: Self = Self(0);
    pub(crate) const ADJACENT: Self = Self(1000);
    pub(crate) const DIAGONAL: Self = Self(1414);
    pub(crate) const VERTICAL: Self = Self(1800);

    pub(crate) fn f32(&self) -> f32 {
        0.001 * self.0 as f32
    }
}

impl Add<Self> for Distance {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Div<Speed> for Distance {
    type Output = Duration;

    fn div(self, speed: Speed) -> Duration {
        Duration(self.0 * 1000 / speed.0)
    }
}
