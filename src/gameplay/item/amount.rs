use crate::gameplay::Fragment;
use crate::hud::HARD_TEXT_COLOR;
use bevy::prelude::Component;
use std::ops::{Add, Sub};

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Amount(pub(crate) u32);

impl Amount {
    pub(crate) const ZERO: Self = Self(0);
    pub(crate) const SINGLE: Self = Self(1);

    pub(crate) fn fragment(&self) -> Option<Fragment> {
        (Self::SINGLE != *self)
            .then_some(Fragment::colorized(format!("{}", self.0), HARD_TEXT_COLOR))
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
