use bevy::prelude::Component;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Millimeter(pub(crate) u64);

impl Millimeter {
    pub(crate) const ADJACENT: Self = Self(1000);
    pub(crate) const DIAGONAL: Self = Self(1414);
    pub(crate) const VERTICAL: Self = Self(1800);

    pub(crate) fn f32(&self) -> f32 {
        0.001 * self.0 as f32
    }
}

#[derive(Clone, Copy)]
pub(crate) struct MillimeterPerSecond(pub(crate) u64);

impl MillimeterPerSecond {
    pub(crate) const fn from_kmph(n: u64) -> Self {
        Self(n * 1_000_000 / 3_600)
    }
}

impl std::fmt::Display for MillimeterPerSecond {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:.00?} km/h", self.0 as f32 * 3_600.0 / 1_000_000.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Milliseconds(pub(crate) u64);

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

#[derive(Clone, Copy)]
pub(crate) struct Distance {
    pub(crate) horizontal: Millimeter,
    pub(crate) up: Millimeter,
    pub(crate) down: Millimeter,
}

impl Distance {
    pub(crate) fn equivalent(&self) -> Millimeter {
        Millimeter(self.horizontal.0 + 2 * self.up.0 + self.down.0)
    }
}

#[derive(Component, Clone, Copy)]
pub(crate) struct Speed {
    pub(crate) h: MillimeterPerSecond,
    pub(crate) up: MillimeterPerSecond,
    pub(crate) down: MillimeterPerSecond,
}

impl Speed {
    pub(crate) const fn from_h_kmph(s: u64) -> Self {
        let h = MillimeterPerSecond::from_kmph(s);
        Self {
            h,
            up: MillimeterPerSecond(2 * h.0),
            down: h,
        }
    }

    pub(crate) fn stay(&self) -> Milliseconds {
        Millimeter(Millimeter::ADJACENT.0 / 2) / self.h
    }

    pub(crate) fn activate(&self) -> Milliseconds {
        Millimeter(3 * Millimeter::ADJACENT.0) / self.h
    }
}

impl std::ops::Div<Speed> for Distance {
    type Output = Milliseconds;

    fn div(self, speed: Speed) -> Milliseconds {
        self.horizontal / speed.h + self.up / speed.up + self.down / speed.down
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Partial(u8);

impl Partial {
    pub(crate) const fn from_u8(from: u8) -> Self {
        Self(from)
    }
}
