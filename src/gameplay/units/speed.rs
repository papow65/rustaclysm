use super::{Distance, Duration};
use crate::prelude::{MoveCost, NborDistance};
use std::{fmt, ops::Add, ops::Mul};

#[derive(Clone, Copy)]
pub(crate) struct Speed {
    millimeter_per_second: u64,
}

impl Speed {
    pub(crate) const fn from_kmph(n: u64) -> Self {
        Self {
            millimeter_per_second: n * 1_000_000 / 3_600,
        }
    }

    pub(crate) const fn millimeter_per_second(&self) -> u64 {
        self.millimeter_per_second
    }
}

impl fmt::Debug for Speed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self}") // use Display
    }
}

impl fmt::Display for Speed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:.01?} km/h",
            self.millimeter_per_second as f32 * 3_600.0 / 1_000_000.0
        )
    }
}

impl Mul<f32> for Speed {
    type Output = Self;

    fn mul(self, value: f32) -> Self {
        Self {
            millimeter_per_second: (self.millimeter_per_second as f32 * value) as u64,
        }
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
