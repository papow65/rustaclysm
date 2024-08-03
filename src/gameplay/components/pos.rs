use crate::{
    gameplay::HorizontalDirection,
    prelude::{LevelOffset, Location, Nbor, PosOffset, VisionDistance, ZoneLevelEntities},
};
use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    prelude::{Component, Vec3},
};
use std::{cmp::Ordering, fmt, iter::once, ops::Sub};

/// Does not include 'from', but does include 'to'
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
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub(crate) struct Level {
    pub(crate) h: i8,
}

impl Level {
    pub(crate) const AMOUNT: usize = 21;
    pub(crate) const GROUND_AMOUNT: usize = (Self::AMOUNT - 1) / 2 + 1;

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
    pub(crate) const GROUNDS: [Self; Self::GROUND_AMOUNT] = [
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

    pub(crate) const fn compare_to_ground(&self) -> Ordering {
        if self.h == 0 {
            Ordering::Equal
        } else if self.h < 0 {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }

    #[inline]
    pub(crate) fn f32(&self) -> f32 {
        (*self - Self::ZERO).f32()
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

// Manually deriving `Component`
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

    pub(crate) const fn horizontal_offset(self, x_offset: i32, z_offset: i32) -> Self {
        Self::new(self.x + x_offset, self.level, self.z + z_offset)
    }

    pub(crate) fn offset(self, offset: PosOffset) -> Option<Self> {
        self.level
            .offset(offset.level)
            .map(|level| Self::new(self.x + offset.x, level, self.z + offset.z))
    }

    pub(crate) const fn horizontal_nbor(self, direction: HorizontalDirection) -> Self {
        let (x, z) = direction.offset();
        Self {
            x: self.x + x,
            level: self.level,
            z: self.z + z,
        }
    }

    /// Get nbor while ignoring stairs - meant for meta operations like examining
    pub(crate) fn raw_nbor(self, nbor: Nbor) -> Option<Self> {
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
            horizontal => Some(self.horizontal_nbor(horizontal.horizontal_projection())),
        }
    }

    #[inline]
    pub(crate) fn vec3(self) -> Vec3 {
        (self - Self::ORIGIN).vec3()
    }

    /// Doe not include 'self', but includes 'to'
    pub(crate) fn straight(self, to: Self) -> impl Iterator<Item = Self> {
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
    pub(crate) fn vision_distance(self, other: Self) -> VisionDistance {
        VisionDistance::from(self - other)
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
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        Location::register_hooks(hooks);
    }
}

#[derive(Component, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SubzoneLevel {
    pub(crate) x: i32,
    pub(crate) level: Level,
    pub(crate) z: i32,
}

impl SubzoneLevel {
    pub(crate) const SIZE: i32 = 12;

    /// CDDA index in map
    pub(crate) const fn index(&self) -> usize {
        2 * (self.x as usize % 2) + (self.z as usize % 2)
    }

    /// CDDA coordinates
    pub(crate) const fn coordinates(&self) -> (i32, i32, i8) {
        (self.x, self.z, self.level.h)
    }

    pub(crate) const fn base_corner(&self) -> Pos {
        Pos::new(Self::SIZE * self.x, self.level, Self::SIZE * self.z)
    }

    pub(crate) const fn corners(&self) -> [Pos; 4] {
        let base_corner = self.base_corner();
        [
            base_corner,
            base_corner.horizontal_offset(0, 11),
            base_corner.horizontal_offset(11, 0),
            base_corner.horizontal_offset(11, 11),
        ]
    }
}

impl fmt::Debug for SubzoneLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SubzoneLevel{{x: {}, {:?}, z: {}}}",
            self.x, self.level, self.z
        )
    }
}

impl From<Pos> for SubzoneLevel {
    fn from(pos: Pos) -> Self {
        Self {
            x: pos.x.div_euclid(Self::SIZE),
            level: pos.level,
            z: pos.z.div_euclid(Self::SIZE),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

impl fmt::Debug for Zone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Zone{{x: {}, z: {}}}", self.x, self.z)
    }
}

impl From<Pos> for Zone {
    fn from(pos: Pos) -> Self {
        Self {
            x: pos.x.div_euclid(Self::SIZE),
            z: pos.z.div_euclid(Self::SIZE),
        }
    }
}

// Manually deriving `Component`
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
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

    pub(crate) fn nbor(self, nbor: Nbor) -> Option<Self> {
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
                let (x, z) = horizontal.horizontal_projection().offset();
                Some(Self {
                    zone: self.zone.offset(x, z),
                    level: self.level,
                })
            }
        }
    }

    pub(crate) const fn base_corner(&self) -> Pos {
        Pos::new(
            Zone::SIZE * self.zone.x,
            self.level,
            Zone::SIZE * self.zone.z,
        )
    }

    pub(crate) const fn center_pos(&self) -> Pos {
        self.base_corner().horizontal_offset(11, 11)
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

impl fmt::Debug for ZoneLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ZoneLevel{{x: {}, {:?}, z: {}}}",
            self.zone.x, self.level, self.zone.z
        )
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

impl Component for ZoneLevel {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        ZoneLevelEntities::register_hooks(hooks);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

impl fmt::Debug for Overzone {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Overzone{{x: {}, {}}}", self.x, self.z)
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
