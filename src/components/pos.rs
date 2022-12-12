use crate::prelude::{
    LevelOffset, Millimeter, Nbor, PosOffset, WalkingDistance, MIN_INVISIBLE_DISTANCE,
};
use bevy::prelude::{Component, Vec3};
use std::{iter::once, ops::Sub};

/** Does not include 'from', but does include 'to' */
fn straight_2d(from: (i32, i32), to: (i32, i32)) -> impl Iterator<Item = (i32, i32)> {
    bresenham::Bresenham::new(
        (from.0 as isize, from.1 as isize),
        (to.0 as isize, to.1 as isize),
    )
    .skip(1) // skip 'from'
    .map(|p| (p.0 as i32, p.1 as i32))
    .chain(once(to))
}

/// Vertical aspect
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Level {
    pub(crate) h: i8,
}

impl Level {
    pub(crate) const AMOUNT: usize = 21;
    pub(crate) const ZERO: Self = Self::new(0);
    const LOWEST: Self = Self::new(-10);
    const HIGHEST: Self = Self::new(10);
    pub(crate) const ALL: [Self; Self::AMOUNT] = [
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
        Self::new(0),
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

    pub(crate) const fn new(level: i8) -> Self {
        Self { h: level }
    }

    fn in_bounds(&self) -> bool {
        &Self::LOWEST <= self && self <= &Self::HIGHEST
    }

    pub(crate) fn up(&self) -> Option<Self> {
        let up = Self { h: self.h + 1 };
        up.in_bounds().then_some(up)
    }

    pub(crate) fn down(&self) -> Option<Self> {
        let down = Self { h: self.h - 1 };
        down.in_bounds().then_some(down)
    }

    pub(crate) fn offset(&self, offset: LevelOffset) -> Option<Self> {
        let sum = Self {
            h: self.h + offset.h,
        };
        sum.in_bounds().then_some(sum)
    }

    pub(crate) const fn dist(self, to: Self) -> u8 {
        self.h.abs_diff(to.h)
    }

    pub(crate) const fn index(&self) -> usize {
        (self.h + 10) as usize
    }

    pub(crate) fn f32(&self) -> f32 {
        f32::from(self.h) as f32 * Millimeter::VERTICAL.f32()
    }
}

impl Sub<Level> for Level {
    type Output = LevelOffset;

    fn sub(self, other: Level) -> LevelOffset {
        LevelOffset {
            h: self.h - other.h,
        }
    }
}

#[derive(Component, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Pos {
    pub(crate) x: i32,
    pub(crate) level: Level,
    pub(crate) z: i32,
}

impl Pos {
    pub(crate) const ORIGIN: Self = Self {
        x: 0,
        level: Level::ZERO,
        z: 0,
    };

    pub(crate) const fn new(x: i32, level: Level, z: i32) -> Self {
        Self { x, level, z }
    }

    /** Distance without regard for obstacles or stairs */
    pub(crate) fn walking_distance(&self, other: Self) -> WalkingDistance {
        let dx = u64::from(self.x.abs_diff(other.x));
        let dy = self.level.h - other.level.h;
        let dz = u64::from(self.z.abs_diff(other.z));

        WalkingDistance {
            horizontal: Millimeter(
                std::cmp::max(dx, dz) * Millimeter::ADJACENT.0
                    + std::cmp::min(dx, dz) * (Millimeter::DIAGONAL.0 - Millimeter::ADJACENT.0),
            ),
            up: Millimeter(if 0 < dy {
                Millimeter::VERTICAL.0 * dy as u64
            } else {
                0
            }),
            down: Millimeter(if dy < 0 {
                Millimeter::VERTICAL.0 * u64::from(dy.unsigned_abs())
            } else {
                0
            }),
        }
    }

    pub(crate) fn offset(self, offset: PosOffset) -> Option<Self> {
        self.level
            .offset(offset.level)
            .map(|level| Self::new(self.x + offset.x, level, self.z + offset.z))
    }

    /// Get nbor while ignoring stairs - meant for meta operations like examining
    pub(crate) fn raw_nbor(self, nbor: &Nbor) -> Option<Self> {
        match nbor {
            Nbor::Up => self.level.up().map(|level| Self {
                x: self.x,
                level,
                z: self.z,
            }),
            Nbor::Down => self.level.down().map(|level| Self {
                x: self.x,
                level,
                z: self.z,
            }),
            horizontal => {
                let (x, z) = horizontal.horizontal_offset();
                Some(Self {
                    x: self.x + x,
                    level: self.level,
                    z: self.z + z,
                })
            }
        }
    }

    pub(crate) fn vec3(self) -> Vec3 {
        Vec3::new(
            f64::from(self.x) as f32 * Millimeter::ADJACENT.f32(),
            self.level.f32(),
            f64::from(self.z) as f32 * Millimeter::ADJACENT.f32(),
        )
    }

    /** Doe not include 'self', but includes 'to' */
    pub(crate) fn straight(self, to: Self) -> impl Iterator<Item = Self> {
        assert!(self != to);

        let max_diff = self
            .x
            .abs_diff(to.x)
            .max(u32::from(self.level.dist(to.level)))
            .max(self.z.abs_diff(to.z)) as i32;
        straight_2d((self.x, 0), (to.x, max_diff))
            .zip(straight_2d(
                (i32::from(self.level.h), 0),
                (i32::from(to.level.h), max_diff),
            ))
            .zip(straight_2d((self.z, 0), (to.z, max_diff)))
            .map(|(((x, _), (y, _)), (z, _))| Self::new(x, Level::new(y as i8), z))
    }

    /** Without regard of obstacles */
    pub(crate) const fn in_visible_range(self, other: Self) -> bool {
        Millimeter::ADJACENT.0.pow(2) * (self.x.abs_diff(other.x) as u64).pow(2)
            + Millimeter::VERTICAL.0.pow(2) * (self.level.h.abs_diff(other.level.h) as u64).pow(2)
            + Millimeter::ADJACENT.0.pow(2) * (self.z.abs_diff(other.z) as u64).pow(2)
            < Millimeter::ADJACENT.0.pow(2) * (MIN_INVISIBLE_DISTANCE as u64).pow(2)
    }
}

impl Sub<Pos> for Pos {
    type Output = PosOffset;

    fn sub(self, other: Pos) -> PosOffset {
        PosOffset {
            x: self.x - other.x,
            level: self.level - other.level,
            z: self.z - other.z,
        }
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SubzoneLevel {
    pub(crate) x: i32,
    pub(crate) level: Level,
    pub(crate) z: i32,
}

impl SubzoneLevel {
    pub(crate) const SIZE: i32 = 12;

    /** CDDA index in map */
    pub(crate) const fn index(&self) -> usize {
        2 * (self.x as usize % 2) + (self.z as usize % 2)
    }

    /** CDDA coordinates */
    pub(crate) const fn coordinates(&self) -> (i32, i32, i32) {
        (self.x, self.z, self.level.h as i32)
    }

    pub(crate) const fn base_pos(&self) -> Pos {
        Pos::new(
            SubzoneLevel::SIZE * self.x,
            self.level,
            SubzoneLevel::SIZE * self.z,
        )
    }
}

impl From<Pos> for SubzoneLevel {
    fn from(pos: Pos) -> Self {
        Self {
            x: pos.x.div_euclid(SubzoneLevel::SIZE),
            level: pos.level,
            z: pos.z.div_euclid(SubzoneLevel::SIZE),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct Zone {
    pub(crate) x: i32,
    pub(crate) z: i32,
}

impl Zone {
    pub(crate) const SIZE: i32 = 2 * SubzoneLevel::SIZE;

    pub(crate) const fn zone_level(&self, level: Level) -> ZoneLevel {
        ZoneLevel { zone: *self, level }
    }

    pub(crate) const fn offset(&self, x: i32, z: i32) -> Self {
        Self {
            x: self.x + x,
            z: self.z + z,
        }
    }
}

impl From<Pos> for Zone {
    fn from(pos: Pos) -> Self {
        Self {
            x: pos.x.div_euclid(Self::SIZE as i32),
            z: pos.z.div_euclid(Self::SIZE as i32),
        }
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ZoneLevel {
    pub(crate) zone: Zone,
    pub(crate) level: Level,
}

impl ZoneLevel {
    pub(crate) fn offset(self, offset: LevelOffset) -> Option<Self> {
        self.level.offset(offset).map(|level| Self {
            zone: self.zone,
            level,
        })
    }

    pub(crate) fn nbor(self, nbor: &Nbor) -> Option<Self> {
        match nbor {
            Nbor::Up => self.level.up().map(|level| Self {
                zone: self.zone,
                level,
            }),
            Nbor::Down => self.level.down().map(|level| Self {
                zone: self.zone,
                level,
            }),
            horizontal => {
                let (x, z) = horizontal.horizontal_offset();
                Some(Self {
                    zone: self.zone.offset(x, z),
                    level: self.level,
                })
            }
        }
    }

    pub(crate) const fn base_pos(&self) -> Pos {
        Pos::new(
            Zone::SIZE * self.zone.x,
            self.level,
            Zone::SIZE * self.zone.z,
        )
    }

    pub(crate) const fn subzone_levels(&self) -> [SubzoneLevel; 4] {
        [
            SubzoneLevel {
                x: 2 * self.zone.x,
                level: self.level,
                z: 2 * self.zone.z,
            },
            SubzoneLevel {
                x: 2 * self.zone.x,
                level: self.level,
                z: 2 * self.zone.z + 1,
            },
            SubzoneLevel {
                x: 2 * self.zone.x + 1,
                level: self.level,
                z: 2 * self.zone.z,
            },
            SubzoneLevel {
                x: 2 * self.zone.x + 1,
                level: self.level,
                z: 2 * self.zone.z + 1,
            },
        ]
    }
}

impl From<Pos> for ZoneLevel {
    fn from(pos: Pos) -> Self {
        Self {
            zone: Zone::from(pos),
            level: pos.level,
        }
    }
}

impl From<SubzoneLevel> for ZoneLevel {
    fn from(subzone_level: SubzoneLevel) -> Self {
        Self {
            zone: Zone {
                x: subzone_level.x.div_euclid(2),
                z: subzone_level.z.div_euclid(2),
            },
            level: subzone_level.level,
        }
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct Overzone {
    pub(crate) x: i32,
    pub(crate) z: i32,
}

impl Overzone {
    pub(crate) const fn base_zone(&self) -> Zone {
        Zone {
            x: 180 * self.x,
            z: 180 * self.z,
        }
    }
}

impl From<Zone> for Overzone {
    fn from(zone: Zone) -> Self {
        Self {
            x: zone.x.div_euclid(180),
            z: zone.z.div_euclid(180),
        }
    }
}
