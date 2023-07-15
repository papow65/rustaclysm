use crate::prelude::{MoveCost, NborDistance};
use pathfinding::num_traits::Zero;
use serde::Deserialize;
use std::{
    fmt,
    iter::Sum,
    ops::{Add, AddAssign, Div, Sub},
};

pub(crate) const MAX_VISIBLE_DISTANCE: i32 = 60;

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

impl Add<Self> for Millimeter {
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

impl fmt::Debug for MillimeterPerSecond {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.01?} km/h", self.0 as f32 * 3_600.0 / 1_000_000.0)
    }
}

impl fmt::Display for MillimeterPerSecond {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.01?} km/h", self.0 as f32 * 3_600.0 / 1_000_000.0)
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Milliseconds(pub(crate) u64);

impl Milliseconds {
    pub(crate) const MINUTE: Self = Self(60 * 1000);
    pub(crate) const EIGHT_HOURS: Self = Self(8 * 60 * 60 * 1000);
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

impl AddAssign for Milliseconds {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl Sub for Milliseconds {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Timestamp(Milliseconds);

impl Timestamp {
    pub(crate) const fn minute_of_day(&self) -> u64 {
        self.0 .0 / (1000 * 60) % (24 * 60)
    }

    /** days, hours, minutes, seconds, deciseconds */
    pub(crate) const fn day_and_time(&self) -> (u64, u8, u8, u8, u8) {
        let tenth_seconds = self.0 .0 / 100;
        let seconds = tenth_seconds / 10;
        let minutes = seconds / 60;
        let hours = minutes / 60;
        (
            hours / 24,
            (hours % 24) as u8,
            (minutes % 60) as u8,
            (seconds % 60) as u8,
            (tenth_seconds % 10) as u8,
        )
    }
}

impl Add<Milliseconds> for Timestamp {
    type Output = Self;

    fn add(self, other: Milliseconds) -> Self {
        Self(self.0 + other)
    }
}

impl AddAssign<Milliseconds> for Timestamp {
    fn add_assign(&mut self, other: Milliseconds) {
        self.0 += other;
    }
}

impl Sub for Timestamp {
    type Output = Milliseconds;

    fn sub(self, other: Self) -> Milliseconds {
        self.0 - other.0
    }
}

impl From<u64> for Timestamp {
    fn from(turn: u64) -> Self {
        Self(Milliseconds(1000 * turn))
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct WalkingCost {
    /** Contains the move cost of every step and double cost for going up */
    equivalent_distance: Millimeter,
}

impl WalkingCost {
    pub(crate) fn new(nbor_distance: &NborDistance, move_cost: MoveCost) -> Self {
        let mut new = Self {
            equivalent_distance: match nbor_distance {
                NborDistance::Up | NborDistance::Down => Millimeter::VERTICAL,
                NborDistance::Adjacent => Millimeter::ADJACENT,
                NborDistance::Diagonal => Millimeter::DIAGONAL,
                NborDistance::Zero => Millimeter::ZERO,
            },
        };
        new.equivalent_distance.0 *= u64::from(move_cost.0);
        if nbor_distance != &NborDistance::Up {
            // 2 is both the penalty for muving up and default move cost.
            new.equivalent_distance.0 /= 2;
        }

        new
    }

    pub(crate) fn duration(&self, speed: MillimeterPerSecond) -> Milliseconds {
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

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, PartialOrd)]
#[serde(from = "String")]
pub(crate) struct Volume {
    milliliter: u64,
}

impl Volume {
    pub(crate) const ZERO: Self = Self { milliliter: 0 };
}

impl Add<Self> for Volume {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            milliliter: self.milliliter + other.milliliter,
        }
    }
}

impl Sub<Self> for Volume {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            milliliter: self.milliliter - other.milliliter,
        }
    }
}

impl fmt::Display for Volume {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if 10_000 <= self.milliliter {
            write!(f, "{} l", self.milliliter / 1000)
        } else if 1_000 <= self.milliliter {
            write!(f, "{:.1} l", self.milliliter as f32 * 0.001)
        } else {
            write!(f, "{} ml", self.milliliter)
        }
    }
}

impl From<String> for Volume {
    fn from(value: String) -> Self {
        let quantity = value.trim_matches(char::is_alphabetic).trim();
        let unit: String = value.matches(char::is_alphabetic).collect();
        //println!("{value} {} {}", &quantity, &unit);

        let quantity = quantity.parse::<f32>().unwrap();

        Self {
            milliliter: match unit.to_lowercase().as_str() {
                "l" => 1_000.0 * quantity,
                "ml" => quantity,
                _ => panic!("{value} {quantity} {}", &unit),
            } as u64,
        }
    }
}

#[cfg(test)]
mod volume_tests {
    use super::*;
    #[test]
    fn it_works() {
        assert_eq!(
            Volume::from(String::from("21 ml")),
            Volume { milliliter: 21 }
        );
        assert_eq!(
            Volume::from(String::from("35.6L")),
            Volume { milliliter: 35_600 }
        );
    }
}

impl Sum for Volume {
    fn sum<V>(iter: V) -> Self
    where
        V: Iterator<Item = Self>,
    {
        iter.fold(Self { milliliter: 0 }, |a, b| Self {
            milliliter: a.milliliter + b.milliliter,
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd)]
#[serde(from = "String")]
pub(crate) struct Mass {
    milligram: u64,
}

impl Mass {
    pub(crate) const ZERO: Self = Self { milligram: 0 };
}

impl Add<Self> for Mass {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            milligram: self.milligram + other.milligram,
        }
    }
}

impl Sub<Self> for Mass {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            milligram: self.milligram - other.milligram,
        }
    }
}

impl fmt::Display for Mass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if 10_000_000 <= self.milligram {
            write!(f, "{} kg", self.milligram / 1_000_000)
        } else if 1_000_000 <= self.milligram {
            write!(f, "{:.1} kg", self.milligram as f32 * 0.000_001)
        } else if 1_000 <= self.milligram {
            write!(f, "{} g", self.milligram / 1000)
        } else {
            write!(f, "{} mg", self.milligram)
        }
    }
}

impl From<String> for Mass {
    fn from(value: String) -> Self {
        let quantity = value.trim_matches(char::is_alphabetic).trim();
        let unit: String = value.matches(char::is_alphabetic).collect();
        //println!("{value} {} {}", &quantity, &unit);

        let quantity = quantity.parse::<f32>().unwrap();

        Self {
            milligram: match unit.to_lowercase().as_str() {
                "mg" => quantity,
                "g" => 1_000.0 * quantity,
                "kg" => 1_000_000.0 * quantity,
                _ => panic!("{value} {quantity} {}", &unit),
            } as u64,
        }
    }
}

#[cfg(test)]
mod mass_tests {
    use super::*;
    #[test]
    fn it_works() {
        assert_eq!(Mass::from(String::from("21mg")), Mass { milligram: 21 });
        assert_eq!(
            Mass::from(String::from("35.6 Kg")),
            Mass {
                milligram: 35_600_000
            }
        );
    }
}
