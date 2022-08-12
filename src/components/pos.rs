use crate::prelude::{Distance, Millimeter, Milliseconds, ADJACENT, DIAGONAL, VERTICAL};
use bevy::prelude::{Component, Vec3};
use pathfinding::prelude::astar;

fn straight_2d(from: (i32, i32), to: (i32, i32)) -> impl Iterator<Item = (i32, i32)> {
    bresenham::Bresenham::new(
        (from.0 as isize, from.1 as isize),
        (to.0 as isize, to.1 as isize),
    )
    .skip(1) // skip 'self'
    .map(|p| (p.0 as i32, p.1 as i32))
    .chain(std::iter::once(to))
}

/// Vertical aspect
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
pub struct Level {
    pub h: i8,
}

impl Level {
    pub const AMOUNT: usize = 21;
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

    pub const fn new(level: i8) -> Self {
        Self { h: level }
    }

    fn in_bounds(&self) -> bool {
        &Self::LOWEST <= self && self <= &Self::HIGHEST
    }

    pub fn up(&self) -> Option<Self> {
        let up = Self { h: self.h + 1 };
        up.in_bounds().then_some(up)
    }

    pub fn down(&self) -> Option<Self> {
        let down = Self { h: self.h - 1 };
        down.in_bounds().then_some(down)
    }

    pub fn offset(&self, relative: Self) -> Option<Self> {
        let sum = Self {
            h: self.h + relative.h,
        };
        sum.in_bounds().then_some(sum)
    }

    pub const fn dist(self, to: Self) -> i8 {
        (self.h - to.h).abs()
    }

    pub const fn index(&self) -> usize {
        (self.h + 10) as usize
    }

    pub fn f32(&self) -> f32 {
        f32::from(self.h) as f32 * VERTICAL.f32()
    }

    pub fn visible_from(&self, reference: Self) -> bool {
        *self == reference || (Self::ZERO <= *self && *self < reference)
    }
}

/// Y is vertical, like the bevy default
#[derive(Component, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Pos {
    pub x: i32,
    pub level: Level,
    pub z: i32,
}

impl Pos {
    pub const fn new(x: i32, level: Level, z: i32) -> Self {
        Self { x, level, z }
    }

    /** Distance without regard for obstacles or stairs */
    pub fn dist(&self, other: Self) -> Distance {
        let dx = u64::from((self.x - other.x).unsigned_abs());
        let dy = self.level.h - other.level.h;
        let dz = u64::from((self.z - other.z).unsigned_abs());

        Distance {
            h: Millimeter(
                std::cmp::max(dx, dz) * ADJACENT.0
                    + std::cmp::min(dx, dz) * (DIAGONAL.0 - ADJACENT.0),
            ),
            up: Millimeter(if 0 < dy { VERTICAL.0 * dy as u64 } else { 0 }),
            down: Millimeter(if dy < 0 {
                VERTICAL.0 * u64::from(dy.unsigned_abs())
            } else {
                0
            }),
        }
    }

    pub fn potential_nbors(self) -> impl Iterator<Item = (Self, Distance)> {
        (0..10)
            .filter_map(move |i| match i {
                0 => Some(Self::new(self.x + 1, self.level, self.z)),
                1 => Some(Self::new(self.x + 1, self.level, self.z + 1)),
                2 => Some(Self::new(self.x, self.level, self.z + 1)),
                3 => Some(Self::new(self.x - 1, self.level, self.z + 1)),
                4 => Some(Self::new(self.x - 1, self.level, self.z)),
                5 => Some(Self::new(self.x - 1, self.level, self.z - 1)),
                6 => Some(Self::new(self.x, self.level, self.z - 1)),
                7 => Some(Self::new(self.x + 1, self.level, self.z - 1)),
                8 => self.level.up().map(|up| Self::new(self.x, up, self.z)),
                _ => self
                    .level
                    .down()
                    .map(|down| Self::new(self.x, down, self.z)),
            })
            .map(move |p| (p, self.dist(p)))
    }

    pub fn is_potential_nbor(self, other: Self) -> bool {
        self.potential_nbors().any(|(p, _)| p == other)
    }

    pub fn offset(self, relative: Self) -> Option<Self> {
        self.level
            .offset(relative.level)
            .map(|level| Self::new(self.x + relative.x, level, self.z + relative.z))
    }

    pub fn vec3(self) -> Vec3 {
        Vec3::new(
            f64::from(self.x) as f32 * ADJACENT.f32(),
            self.level.f32(),
            f64::from(self.z) as f32 * ADJACENT.f32(),
        )
    }

    pub fn straight(self, to: Self) -> impl Iterator<Item = Self> {
        assert!(self != to);

        let max_diff = (self.x - to.x)
            .abs()
            .max(i32::from(self.level.dist(to.level)))
            .max((self.z - to.z).abs());
        straight_2d((self.x, 0), (to.x, max_diff))
            .zip(straight_2d(
                (i32::from(self.level.h), 0),
                (i32::from(to.level.h), max_diff),
            ))
            .zip(straight_2d((self.z, 0), (to.z, max_diff)))
            .map(|(((x, _), (y, _)), (z, _))| Self::new(x, Level::new(y as i8), z))
    }
}

#[derive(Debug)]
pub struct Path {
    pub first: Pos,
    pub duration: Milliseconds,
    pub destination: Pos,
}

impl Path {
    pub fn plan<FN, IN, FH>(
        from: Pos,
        successors: FN,
        heuristic: FH,
        destination: Pos,
    ) -> Option<Self>
    where
        FN: FnMut(&Pos) -> IN,
        IN: Iterator<Item = (Pos, Milliseconds)>,
        FH: FnMut(&Pos) -> Milliseconds,
    {
        if let Some((mut steps, duration)) =
            astar(&from, successors, heuristic, |&pos| pos == destination)
        {
            assert!(!steps.is_empty());
            assert!(steps.remove(0) == from);
            assert!(!steps.is_empty());
            Some(Self {
                first: *steps.first().unwrap(),
                duration,
                destination,
            })
        } else {
            None
        }
    }

    pub fn improvize<I, FH>(nbors: I, mut heuristic: FH, destination: Pos) -> Option<Self>
    where
        I: Iterator<Item = (Pos, Milliseconds)>,
        FH: FnMut(&Pos) -> Milliseconds,
    {
        nbors
            .map(|(first, duration)| Self {
                first,
                duration: duration + heuristic(&first),
                destination,
            })
            .min_by_key(|path| path.duration)
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Zone {
    pub x: i32,
    pub z: i32,
}

impl Zone {
    pub const fn zone_level(&self, level: Level) -> ZoneLevel {
        ZoneLevel {
            x: self.x,
            level,
            z: self.z,
        }
    }

    pub fn dist(&self, other: Self) -> u32 {
        (self.x - other.x).abs().max((self.z - other.z).abs()) as u32
    }

    pub const fn offset(&self, x: i32, z: i32) -> Self {
        Self {
            x: self.x + x,
            z: self.z + z,
        }
    }

    pub fn nearby(&self, n: u32) -> Vec<Self> {
        let n = i32::try_from(n).unwrap();
        (-n..n)
            .flat_map(move |x| (-n..n).map(move |z| self.offset(x, z)))
            .collect()
    }
}

impl From<Pos> for Zone {
    fn from(pos: Pos) -> Self {
        Self {
            x: pos.x.div_euclid(24),
            z: pos.z.div_euclid(24),
        }
    }
}

impl From<ZoneLevel> for Zone {
    fn from(zone_level: ZoneLevel) -> Self {
        Self {
            x: zone_level.x,
            z: zone_level.z,
        }
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ZoneLevel {
    pub x: i32,
    pub level: Level,
    pub z: i32,
}

impl ZoneLevel {
    pub fn offset(self, relative: Self) -> Option<Self> {
        self.level.offset(relative.level).map(|level| Self {
            x: self.x + relative.x,
            level,
            z: self.z + relative.z,
        })
    }

    pub const fn base_pos(&self) -> Pos {
        Pos::new(24 * self.x, self.level, 24 * self.z)
    }
}

impl From<Pos> for ZoneLevel {
    fn from(pos: Pos) -> Self {
        Self {
            x: pos.x.div_euclid(24),
            level: pos.level,
            z: pos.z.div_euclid(24),
        }
    }
}
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Overzone {
    pub x: i32,
    pub z: i32,
}

impl Overzone {
    pub const fn base_zone(&self) -> Zone {
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

/** Indication that the player moved to or examined another level */
#[derive(Component)]
pub struct LevelChanged;

/** Indication that the player moved to or examined a new zone */
#[derive(Component)]
pub struct ZoneChanged;
