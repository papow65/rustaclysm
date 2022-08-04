use bevy::prelude::Component;

#[derive(Clone, Copy, Debug)]
pub struct Millimeter(pub u64);

impl Millimeter {
    pub fn f32(&self) -> f32 {
        0.001 * self.0 as f32
    }
}

#[derive(Clone, Copy)]
pub struct MillimeterPerSecond(pub u64);

impl MillimeterPerSecond {
    pub const fn from_kmph(n: u64) -> Self {
        Self(n * 1_000_000 / 3_600)
    }
}

impl std::fmt::Display for MillimeterPerSecond {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:.00?} km/h", self.0 as f32 * 3_600.0 / 1_000_000.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Milliseconds(pub u64);

impl std::fmt::Debug for Milliseconds {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:.03?} s", self.0 as f32 * 0.001)
    }
}

impl std::ops::Add for Milliseconds {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl pathfinding::num_traits::Zero for Milliseconds {
    fn zero() -> Self {
        Self(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl std::ops::Div<MillimeterPerSecond> for Millimeter {
    type Output = Milliseconds;

    fn div(self, speed: MillimeterPerSecond) -> Milliseconds {
        Milliseconds(self.0 * 1000 / speed.0)
    }
}

pub const ADJACENT: Millimeter = Millimeter(1000);
pub const DIAGONAL: Millimeter = Millimeter(1414);
pub const VERTICAL: Millimeter = Millimeter(1800);

#[derive(Clone, Copy)]
pub struct Distance {
    pub h: Millimeter,
    pub up: Millimeter,
    pub down: Millimeter,
}

#[derive(Component, Clone, Copy)]
pub struct Speed {
    pub h: MillimeterPerSecond,
    pub up: MillimeterPerSecond,
    pub down: MillimeterPerSecond,
}

impl Speed {
    pub const fn from_h_kmph(s: u64) -> Self {
        let h = MillimeterPerSecond::from_kmph(s);
        Self {
            h,
            up: MillimeterPerSecond(2 * h.0),
            down: h,
        }
    }

    pub fn stay(&self) -> Milliseconds {
        Millimeter(ADJACENT.0 / 2) / self.h
    }

    pub fn activate(&self) -> Milliseconds {
        Millimeter(3 * ADJACENT.0) / self.h
    }
}

impl std::ops::Div<Speed> for Distance {
    type Output = Milliseconds;

    fn div(self, speed: Speed) -> Milliseconds {
        self.h / speed.h + self.up / speed.up + self.down / speed.down
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Partial(u8);

impl Partial {
    pub const fn from_u8(from: u8) -> Self {
        Self(from)
    }
}
