use super::{Distance, Duration};
use crate::prelude::{MoveCost, NborDistance};
use std::{fmt, ops::Add};

#[derive(Clone, Copy)]
pub(crate) struct Speed(pub(crate) u64);

impl Speed {
    pub(crate) const fn from_kmph(n: u64) -> Self {
        Self(n * 1_000_000 / 3_600)
    }
}

impl fmt::Debug for Speed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.01?} km/h", self.0 as f32 * 3_600.0 / 1_000_000.0)
    }
}

impl fmt::Display for Speed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.01?} km/h", self.0 as f32 * 3_600.0 / 1_000_000.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct WalkingCost {
    /// Contains the move cost of every step and double cost for going up
    equivalent_distance: Distance,
}

impl WalkingCost {
    pub(crate) fn new(nbor_distance: NborDistance, move_cost: MoveCost) -> Self {
        let mut new = Self {
            equivalent_distance: match nbor_distance {
                NborDistance::Up | NborDistance::Down => Distance::VERTICAL,
                NborDistance::Adjacent => Distance::ADJACENT,
                NborDistance::Diagonal => Distance::DIAGONAL,
                NborDistance::Zero => Distance::ZERO,
            },
        };
        new.equivalent_distance.0 *= u64::from(move_cost.value());
        if nbor_distance != NborDistance::Up {
            // 2 is both the penalty for muving up and default move cost.
            new.equivalent_distance.0 /= 2;
        }

        new
    }

    pub(crate) fn duration(&self, speed: Speed) -> Duration {
        self.equivalent_distance / speed
    }

    pub(crate) fn f32(&self) -> f32 {
        1.0 / (self.equivalent_distance.0 as f32)
    }
}

impl Add<Self> for WalkingCost {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            equivalent_distance: self.equivalent_distance + other.equivalent_distance,
        }
    }
}
