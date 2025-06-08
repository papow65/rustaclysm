use crate::{HorizontalDirection, Level, LevelOffset, LocationCache, Nbor, VisionDistance};
use bevy::ecs::component::{ComponentHooks, Immutable, StorageType};
use bevy::prelude::{Component, Vec3};
use bresenham::Bresenham;
use cdda_json_files::At;
use std::{fmt, iter::once, ops::Sub};
use units::Distance;

/// Does not include 'from', but does include 'to'
fn straight_2d(from: (i32, i32), to: (i32, i32)) -> impl Iterator<Item = (i32, i32)> {
    Bresenham::new(
        (from.0 as isize, from.1 as isize),
        (to.0 as isize, to.1 as isize),
    )
    .skip(1) // skip 'from'
    .map(|p| (p.0 as i32, p.1 as i32))
    .chain(once(to))
}

/// The position of characters, items, terrain, furniture, ...
// Manually deriving `Component`
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: i32,
    pub level: Level,
    pub z: i32,
}

impl Pos {
    pub const ORIGIN: Self = Self {
        x: 0,
        level: Level::ZERO,
        z: 0,
    };

    #[must_use]
    pub const fn new(x: i32, level: Level, z: i32) -> Self {
        Self { x, level, z }
    }

    #[must_use]
    pub const fn horizontal_offset(self, x_offset: i32, z_offset: i32) -> Self {
        Self::new(self.x + x_offset, self.level, self.z + z_offset)
    }

    #[must_use]
    pub fn offset(self, offset: PosOffset) -> Option<Self> {
        self.level
            .offset(offset.level)
            .map(|level| Self::new(self.x + offset.x, level, self.z + offset.z))
    }

    #[must_use]
    pub const fn horizontal_nbor(self, direction: HorizontalDirection) -> Self {
        let (x, z) = direction.offset();
        Self {
            x: self.x + x,
            level: self.level,
            z: self.z + z,
        }
    }

    /// Get nbor while ignoring stairs - meant for meta operations like examining
    #[must_use]
    pub fn raw_nbor(self, nbor: Nbor) -> Option<Self> {
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
            Nbor::Horizontal(horizontal_direction) => {
                Some(self.horizontal_nbor(horizontal_direction))
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn vec3(self) -> Vec3 {
        (self - Self::ORIGIN).vec3()
    }

    /// Doe not include 'self', but includes 'to'
    pub fn straight(self, to: Self) -> impl Iterator<Item = Self> {
        assert_ne!(self, to, "The begin and end should be different positions");

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

    /// Without regard for obstacles
    #[must_use]
    pub fn vision_distance(self, other: Self) -> VisionDistance {
        VisionDistance::from_offset(self - other)
    }
}

impl fmt::Debug for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Pos{{x: {}, {:?}, z: {}}}", self.x, self.level, self.z)
    }
}

impl Sub<Self> for Pos {
    type Output = PosOffset;

    fn sub(self, other: Self) -> PosOffset {
        PosOffset {
            x: self.x - other.x,
            level: self.level - other.level,
            z: self.z - other.z,
        }
    }
}

impl Component for Pos {
    type Mutability = Immutable;

    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        LocationCache::register_hooks(hooks);
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct PosOffset {
    pub x: i32,
    pub level: LevelOffset,
    pub z: i32,
}

impl PosOffset {
    pub const HERE: Self = Self {
        x: 0,
        level: LevelOffset::ZERO,
        z: 0,
    };

    #[must_use]
    pub fn player_hint(&self) -> &str {
        if self.x == 0 && self.z == 0 {
            match self.level {
                LevelOffset { h } if h > 0 => "U",
                LevelOffset::ZERO => "H",
                _ => "D",
            }
        } else if 2 * self.z.abs() <= self.x.abs() {
            if 0 < self.x { "E" } else { "W" }
        } else if 2 * self.x.abs() <= self.z.abs() {
            if 0 < self.z { "S" } else { "N" }
        } else if 0 < self.x {
            if 0 < self.z {
                "SE"
            } else {
                assert!(self.z < 0, "Unexpected offset: {self:?}");
                "NE"
            }
        } else {
            assert!(self.x < 0, "Unexpected offset: {self:?}");
            if 0 < self.z {
                "SW"
            } else {
                assert!(self.z < 0, "Unexpected offset: {self:?}");
                "NW"
            }
        }
    }

    #[must_use]
    pub const fn down(&self) -> Self {
        Self {
            level: LevelOffset {
                h: self.level.h - 1,
            },
            ..*self
        }
    }

    #[must_use]
    pub fn vec3(&self) -> Vec3 {
        Vec3::new(
            f64::from(self.x) as f32 * Distance::ADJACENT.meter_f32(),
            self.level.f32(),
            f64::from(self.z) as f32 * Distance::ADJACENT.meter_f32(),
        )
    }

    #[must_use]
    pub const fn get<'a, T>(&'a self, at: &'a At<T>) -> Option<&'a T> {
        if self.x as u8 == at.x && self.z as u8 == at.y {
            Some(&at.obj)
        } else {
            None
        }
    }
}
