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

    pub(crate) fn offset(&self, relative: Self) -> Option<Self> {
        let sum = Self {
            h: self.h + relative.h,
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
        f32::from(self.h) as f32 * VERTICAL.f32()
    }

    pub(crate) fn visible_from(&self, reference: Self) -> bool {
        *self == reference || (Self::ZERO <= *self && *self < reference)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct HorizontalNborOffset {
    /*private*/ x: i32, // -1, 0, or 1
    /*private*/ z: i32, // -1, 0, or 1
}

impl HorizontalNborOffset {
    fn try_from(x: i32, z: i32) -> Option<HorizontalNborOffset> {
        if x.abs().max(z.abs()) == 1 {
            Some(HorizontalNborOffset { x, z })
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Nbor {
    Up,
    Horizontal(HorizontalNborOffset),
    Here,
    Down,
}

impl Nbor {
    pub(crate) const ALL: [Self; 11] = [
        Self::Up,
        Self::Horizontal(HorizontalNborOffset { x: 1, z: 0 }),
        Self::Horizontal(HorizontalNborOffset { x: 1, z: 1 }),
        Self::Horizontal(HorizontalNborOffset { x: 0, z: 1 }),
        Self::Horizontal(HorizontalNborOffset { x: -1, z: 1 }),
        Self::Horizontal(HorizontalNborOffset { x: -1, z: 0 }),
        Self::Horizontal(HorizontalNborOffset { x: -1, z: -1 }),
        Self::Horizontal(HorizontalNborOffset { x: 0, z: -1 }),
        Self::Horizontal(HorizontalNborOffset { x: 1, z: -1 }),
        Self::Here,
        Self::Down,
    ];

    pub(crate) fn try_horizontal(x: i32, z: i32) -> Option<Self> {
        HorizontalNborOffset::try_from(x, z).map(Self::Horizontal)
    }

    pub(crate) const fn horizontal_offset(&self) -> (i32, i32) {
        match self {
            Self::Horizontal(HorizontalNborOffset { x, z }) => (*x, *z),
            _ => (0, 0),
        }
    }

    pub(crate) fn distance(&self) -> Distance {
        match self {
            Self::Up => Distance {
                h: Millimeter(0),
                up: VERTICAL,
                down: Millimeter(0),
            },
            Self::Down => Distance {
                h: Millimeter(0),
                up: Millimeter(0),
                down: VERTICAL,
            },
            horizontal => {
                let (x, z) = horizontal.horizontal_offset();
                Distance {
                    h: if x == 0 || z == 0 { ADJACENT } else { DIAGONAL },
                    up: Millimeter(0),
                    down: Millimeter(0),
                }
            }
        }
    }
}

/// Y is vertical, like the bevy default
#[derive(Component, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Pos {
    pub(crate) x: i32,
    pub(crate) level: Level,
    pub(crate) z: i32,
}

impl Pos {
    pub(crate) const fn new(x: i32, level: Level, z: i32) -> Self {
        Self { x, level, z }
    }

    /** Distance without regard for obstacles or stairs */
    pub(crate) fn dist(&self, other: Self) -> Distance {
        let dx = u64::from(self.x.abs_diff(other.x));
        let dy = self.level.h - other.level.h;
        let dz = u64::from(self.z.abs_diff(other.z));

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

    pub(crate) fn offset(self, relative: Self) -> Option<Self> {
        self.level
            .offset(relative.level)
            .map(|level| Self::new(self.x + relative.x, level, self.z + relative.z))
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
            f64::from(self.x) as f32 * ADJACENT.f32(),
            self.level.f32(),
            f64::from(self.z) as f32 * ADJACENT.f32(),
        )
    }

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
}

#[derive(Debug)]
pub(crate) struct Path {
    pub(crate) first: Pos,
    pub(crate) duration: Milliseconds,
    pub(crate) destination: Pos,
}

impl Path {
    pub(crate) fn plan<FN, IN, FH>(
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

    pub(crate) fn improvize<I, FH>(nbors: I, mut heuristic: FH, destination: Pos) -> Option<Self>
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
pub(crate) struct Zone {
    pub(crate) x: i32,
    pub(crate) z: i32,
}

impl Zone {
    pub(crate) const SIZE: usize = 24;

    pub(crate) const fn zone_level(&self, level: Level) -> ZoneLevel {
        ZoneLevel {
            x: self.x,
            level,
            z: self.z,
        }
    }

    pub(crate) fn dist(&self, other: Self) -> u32 {
        self.x.abs_diff(other.x).max(self.z.abs_diff(other.z))
    }

    pub(crate) const fn offset(&self, x: i32, z: i32) -> Self {
        Self {
            x: self.x + x,
            z: self.z + z,
        }
    }

    pub(crate) fn nearby(&self, n: u32) -> Vec<Self> {
        let n = i32::try_from(n).unwrap();
        (-n..n)
            .flat_map(move |x| (-n..n).map(move |z| self.offset(x, z)))
            .collect()
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

impl From<ZoneLevel> for Zone {
    fn from(zone_level: ZoneLevel) -> Self {
        Self {
            x: zone_level.x,
            z: zone_level.z,
        }
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ZoneLevel {
    pub(crate) x: i32,
    pub(crate) level: Level,
    pub(crate) z: i32,
}

impl ZoneLevel {
    pub(crate) fn offset(self, relative: Self) -> Option<Self> {
        self.level.offset(relative.level).map(|level| Self {
            x: self.x + relative.x,
            level,
            z: self.z + relative.z,
        })
    }

    pub(crate) fn nbor(self, nbor: &Nbor) -> Option<Self> {
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

    pub(crate) const fn base_pos(&self) -> Pos {
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

/** Indication that the player moved to or examined another level */
#[derive(Component)]
pub(crate) struct LevelChanged;

/** Indication that the player moved to or examined a new zone */
#[derive(Component)]
pub(crate) struct ZoneChanged;
