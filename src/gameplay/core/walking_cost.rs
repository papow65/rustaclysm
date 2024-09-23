use crate::gameplay::NborDistance;
use cdda::MoveCost;
use std::iter::Sum;
use std::ops::{Add, Mul};
use units::{Distance, Duration, Speed};

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
        new.equivalent_distance *= u64::from(move_cost.value());
        if nbor_distance != NborDistance::Up {
            // 2 is both the penalty for muving up and default move cost.
            new.equivalent_distance /= 2;
        }

        new
    }

    pub(crate) fn duration(&self, speed: Speed) -> Duration {
        self.equivalent_distance / speed
    }

    pub(crate) fn f32(&self) -> f32 {
        0.001 / self.equivalent_distance.meter_f32()
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

impl Mul<u64> for WalkingCost {
    type Output = Self;

    fn mul(self, other: u64) -> Self {
        Self {
            equivalent_distance: self.equivalent_distance * other,
        }
    }
}

impl Sum<Self> for WalkingCost {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(
            Self {
                equivalent_distance: Distance::ZERO,
            },
            |total, added| total + added,
        )
    }
}

#[cfg(test)]
mod volume_tests {
    use super::*;

    #[test]
    fn parsing_works() {
        assert_eq!(
            WalkingCost::new(NborDistance::Adjacent, MoveCost::default()).f32(),
            0.001
        );
    }
}
