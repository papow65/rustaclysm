use crate::prelude::{MoveCost, NborDistance};
use bevy::prelude::Component;
use pathfinding::num_traits::Zero;
use std::{
    fmt,
    ops::{Add, Div},
};

pub(crate) const MIN_INVISIBLE_DISTANCE: u32 = 61;
/** Approximate, prefer `MIN_INVISIBLE_DISTANCE`  */
pub(crate) const MAX_VISIBLE_DISTANCE: i32 = MIN_INVISIBLE_DISTANCE as i32 - 1;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Millimeter(pub(crate) u64);

impl Millimeter {
    pub(crate) const ZERO: Self = Self(0);
    pub(crate) const ADJACENT: Self = Self(1000);
    pub(crate) const DIAGONAL: Self = Self(1414);
    pub(crate) const VERTICAL: Self = Self(1800);

    pub(crate) fn f32(&self) -> f32 {
        0.001 * self.0 as f32
    }
}

impl Add<Millimeter> for Millimeter {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct MillimeterPerSecond(pub(crate) u64);

impl MillimeterPerSecond {
    pub(crate) const fn from_kmph(n: u64) -> Self {
        Self(n * 1_000_000 / 3_600)
    }
}

impl fmt::Display for MillimeterPerSecond {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.00?} km/h", self.0 as f32 * 3_600.0 / 1_000_000.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Milliseconds(pub(crate) u64);

impl Milliseconds {
    pub(crate) const MINUTE: Milliseconds = Milliseconds(60_000);
}

impl fmt::Debug for Milliseconds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.03?} s", self.0 as f32 * 0.001)
    }
}

impl Add for Milliseconds {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Zero for Milliseconds {
    fn zero() -> Self {
        Self(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl Div<MillimeterPerSecond> for Millimeter {
    type Output = Milliseconds;

    fn div(self, speed: MillimeterPerSecond) -> Milliseconds {
        Milliseconds(self.0 * 1000 / speed.0)
    }
}

#[derive(Component, Clone, Copy)]
pub(crate) struct BaseSpeed(MillimeterPerSecond);

impl BaseSpeed {
    pub(crate) const fn from_h_kmph(s: u64) -> Self {
        Self(MillimeterPerSecond::from_kmph(s))
    }

    pub(crate) fn stay(&self) -> Milliseconds {
        Millimeter(Millimeter::ADJACENT.0 / 2) / self.0
    }

    pub(crate) fn activate(&self) -> Milliseconds {
        Millimeter(3 * Millimeter::ADJACENT.0) / self.0
    }
}

impl fmt::Display for BaseSpeed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Max speed {}", self.0)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Partial(u8);

impl Partial {
    pub(crate) const fn from_u8(from: u8) -> Self {
        Self(from)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct WalkingCost {
    /** Contains the move cost of every step and double cost for going up */
    equivalent_distance: Millimeter,
}

impl WalkingCost {
    pub(crate) fn new(nbor_distance: NborDistance, move_cost: MoveCost) -> Self {
        let mut new = Self {
            equivalent_distance: match nbor_distance {
                NborDistance::Up | NborDistance::Down => Millimeter::VERTICAL,
                NborDistance::Adjacent => Millimeter::ADJACENT,
                NborDistance::Diagonal => Millimeter::DIAGONAL,
                NborDistance::Zero => Millimeter::ZERO,
            },
        };
        new.equivalent_distance.0 *= u64::from(move_cost.0);
        if nbor_distance != NborDistance::Up {
            // 2 is both the penalty for muving up and default move cost.
            new.equivalent_distance.0 /= 2;
        }

        new
    }

    pub(crate) fn duration(&self, speed: BaseSpeed) -> Milliseconds {
        self.equivalent_distance / speed.0
    }

    pub(crate) fn f32(&self) -> f32 {
        1.0 / (self.equivalent_distance.0 as f32)
    }
}

impl Add<WalkingCost> for WalkingCost {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            equivalent_distance: self.equivalent_distance + other.equivalent_distance,
        }
    }
}
