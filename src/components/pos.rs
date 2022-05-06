use bevy::prelude::{Component, Vec3};
use pathfinding::prelude::astar;

use super::super::unit::{Distance, Millimeter, Milliseconds, ADJACENT, DIAGONAL, VERTICAL};

fn straight_2d(from: (i32, i32), to: (i32, i32)) -> impl Iterator<Item = (i32, i32)> {
    bresenham::Bresenham::new(
        (from.0 as isize, from.1 as isize),
        (to.0 as isize, to.1 as isize),
    )
    .skip(1) // skip 'self'
    .map(|p| (p.0 as i32, p.1 as i32))
    .chain(std::iter::once(to))
}

/// Y is vertical
#[derive(Component, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Pos(pub i32, pub i32, pub i32);

impl Pos {
    pub const fn vertical_range() -> core::ops::RangeInclusive<i32> {
        -10..=10
    }

    pub const fn in_bounds(&self) -> bool {
        let vertical_range = Self::vertical_range();
        *vertical_range.start() <= self.1 && self.1 <= *vertical_range.end()
        /*0 <= self.0
        && self.0 < SIZE.0
        &&
        && 0 <= self.2
        && self.2 < SIZE.2*/
    }

    /** Distance without regard for obstacles or stairs */
    pub fn dist(&self, other: Self) -> Distance {
        let dx = (self.0 - other.0).abs() as u64;
        let dy = self.1 - other.1;
        let dz = (self.2 - other.2).abs() as u64;

        Distance {
            h: Millimeter(
                std::cmp::max(dx, dz) * ADJACENT.0
                    + std::cmp::min(dx, dz) * (DIAGONAL.0 - ADJACENT.0),
            ),
            up: Millimeter(if 0 < dy { VERTICAL.0 * dy as u64 } else { 0 }),
            down: Millimeter(if dy < 0 {
                VERTICAL.0 * dy.abs() as u64
            } else {
                0
            }),
        }
    }

    pub fn potential_nbors(self) -> impl Iterator<Item = (Self, Distance)> {
        (0..10)
            .map(move |i| match i {
                0 => Self(self.0 + 1, self.1, self.2),
                1 => Self(self.0 + 1, self.1, self.2 + 1),
                2 => Self(self.0, self.1, self.2 + 1),
                3 => Self(self.0 - 1, self.1, self.2 + 1),
                4 => Self(self.0 - 1, self.1, self.2),
                5 => Self(self.0 - 1, self.1, self.2 - 1),
                6 => Self(self.0, self.1, self.2 - 1),
                7 => Self(self.0 + 1, self.1, self.2 - 1),
                8 => Self(self.0, self.1 + 1, self.2),
                _ => Self(self.0, self.1 - 1, self.2),
            })
            .filter(Self::in_bounds)
            .map(move |p| (p, self.dist(p)))
    }

    pub fn is_potential_nbor(self, other: Self) -> bool {
        self.potential_nbors().any(|(p, _)| p == other)
    }

    pub const fn offset(self, relative: Self) -> Option<Self> {
        let other = Self(
            self.0 + relative.0,
            self.1 + relative.1,
            self.2 + relative.2,
        );
        if other.in_bounds() {
            Some(other)
        } else {
            None
        }
    }

    pub fn vec3(self) -> Vec3 {
        Vec3::new(
            f64::from(self.0) as f32 * ADJACENT.f32(),
            f64::from(self.1) as f32 * VERTICAL.f32(),
            f64::from(self.2) as f32 * ADJACENT.f32(),
        )
    }

    pub fn straight(self, to: Self) -> impl Iterator<Item = Self> {
        assert!(self != to);

        let max_diff = (self.0 - to.0)
            .abs()
            .max((self.1 - to.1).abs())
            .max((self.2 - to.2).abs());
        straight_2d((self.0, 0), (to.0, max_diff))
            .zip(straight_2d((self.1, 0), (to.1, max_diff)))
            .zip(straight_2d((self.2, 0), (to.2, max_diff)))
            .map(|(((x, _), (y, _)), (z, _))| Self(x, y, z))
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
    pub const fn offset(&self, x: i32, z: i32) -> Self {
        Self {
            x: self.x + x,
            z: self.z + z,
        }
    }

    pub const fn zone_level(&self, y: i32) -> ZoneLevel {
        ZoneLevel {
            x: self.x,
            y,
            z: self.z,
        }
    }

    pub fn dist(&self, other: Self) -> u16 {
        (self.x - other.x).abs().max((self.z - other.z).abs()) as u16
    }

    pub fn nearby(&self, n: u8) -> Vec<Self> {
        let n = i32::from(n);
        (-n..n)
            .flat_map(move |x| {
                (-n..n).map(move |z| Self {
                    x: self.x + x,
                    z: self.z + z,
                })
            })
            .collect()
    }
}

impl From<Pos> for Zone {
    fn from(pos: Pos) -> Self {
        Self {
            x: pos.0.div_euclid(24),
            z: pos.2.div_euclid(24),
        }
    }
}

impl From<ZoneLevel> for Zone {
    fn from(zone_level: ZoneLevel) -> Self {
        Self {
            x: zone_level.x,
            z: zone_level.y,
        }
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct ZoneLevel {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ZoneLevel {
    pub const fn offset(&self, x: i32, z: i32) -> Self {
        Self {
            x: self.x + x,
            y: self.y,
            z: self.z + z,
        }
    }

    pub const fn base_pos(&self) -> Pos {
        Pos(24 * self.x, self.y, 24 * self.z)
    }
}

impl From<Pos> for ZoneLevel {
    fn from(pos: Pos) -> Self {
        Self {
            x: pos.0.div_euclid(24),
            y: pos.1,
            z: pos.2.div_euclid(24),
        }
    }
}

#[derive(Component)]
pub struct PosYChanged;

#[derive(Component)]
pub struct ZoneChanged;
