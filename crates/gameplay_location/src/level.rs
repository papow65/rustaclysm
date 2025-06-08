use cdda_json_files::Overmap;
use std::{cmp::Ordering, fmt, ops::Sub};
use units::Distance;

/// Floor level, used as the vertical dimension (Y-axis)
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Level {
    pub h: i8,
}

impl Level {
    pub const AMOUNT: usize = Overmap::LEVEL_AMOUNT;
    pub const GROUND_AMOUNT: usize = (Self::AMOUNT - 1) / 2 + 1;

    pub const ZERO: Self = Self::new(0);
    const LOWEST: Self = Self::new(-10);
    const HIGHEST: Self = Self::new(10);

    pub const ALL: [Self; Self::AMOUNT] = [
        Self::LOWEST,
        Self::new(-9),
        Self::new(-8),
        Self::new(-7),
        Self::new(-6),
        Self::new(-5),
        Self::new(-4),
        Self::new(-3),
        Self::new(-2),
        Self::new(-1),
        Self::ZERO,
        Self::new(1),
        Self::new(2),
        Self::new(3),
        Self::new(4),
        Self::new(5),
        Self::new(6),
        Self::new(7),
        Self::new(8),
        Self::new(9),
        Self::HIGHEST,
    ];
    pub const GROUNDS: [Self; Self::GROUND_AMOUNT] = [
        Self::LOWEST,
        Self::new(-9),
        Self::new(-8),
        Self::new(-7),
        Self::new(-6),
        Self::new(-5),
        Self::new(-4),
        Self::new(-3),
        Self::new(-2),
        Self::new(-1),
        Self::ZERO,
    ];

    #[must_use]
    pub const fn new(level: i8) -> Self {
        Self { h: level }
    }

    #[must_use]
    fn in_bounds(self) -> bool {
        Self::LOWEST <= self && self <= Self::HIGHEST
    }

    #[must_use]
    pub fn up(self) -> Option<Self> {
        let up = Self { h: self.h + 1 };
        up.in_bounds().then_some(up)
    }

    #[must_use]
    pub fn down(self) -> Option<Self> {
        let down = Self { h: self.h - 1 };
        down.in_bounds().then_some(down)
    }

    #[must_use]
    pub(crate) fn offset(self, offset: LevelOffset) -> Option<Self> {
        let sum = Self {
            h: self.h + offset.h,
        };
        sum.in_bounds().then_some(sum)
    }

    #[must_use]
    pub(crate) const fn dist(self, to: Self) -> u8 {
        self.h.abs_diff(to.h)
    }

    #[must_use]
    pub const fn index(self) -> usize {
        (self.h + 10) as usize
    }

    #[must_use]
    pub const fn compare_to_ground(self) -> Ordering {
        if self.h == 0 {
            Ordering::Equal
        } else if self.h < 0 {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }

    #[inline]
    #[must_use]
    pub fn f32(self) -> f32 {
        (self - Self::ZERO).f32()
    }
}

impl fmt::Debug for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Level {}", self.h)
    }
}

impl Sub<Self> for Level {
    type Output = LevelOffset;

    fn sub(self, other: Self) -> LevelOffset {
        LevelOffset {
            h: self.h - other.h,
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct LevelOffset {
    pub h: i8,
}

impl LevelOffset {
    pub const ZERO: Self = Self { h: 0 };
    pub const UP: Self = Self { h: 1 };
    pub const DOWN: Self = Self { h: -1 };

    pub(crate) fn f32(self) -> f32 {
        f32::from(self.h) * Distance::VERTICAL.meter_f32()
    }
}
