use crate::gameplay::Fragment;
use bevy::prelude::Component;
use std::ops::{Add, Sub};

/// Mutable component
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Component)]
pub(crate) struct Amount(pub(crate) u32);

impl Amount {
    pub(crate) const ZERO: Self = Self(0);
    pub(crate) const SINGLE: Self = Self(1);

    pub(crate) fn fragment(&self) -> Option<Fragment> {
        (Self::SINGLE != *self).then_some(Fragment::hard(format!("{}", self.0)))
    }
}

impl Add<Self> for &Amount {
    type Output = Amount;
    fn add(self, other: Self) -> Self::Output {
        Amount(self.0 + other.0)
    }
}

impl Sub<Self> for &Amount {
    type Output = Amount;
    fn sub(self, other: Self) -> Self::Output {
        Amount(self.0 - other.0)
    }
}
