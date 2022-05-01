use bevy::prelude::{Component, Vec3};
use pathfinding::prelude::astar;

use super::super::units::{Distance, Millimeter, Milliseconds, ADJACENT, DIAGONAL, VERTICAL};

fn straight_2d(from: (i16, i16), to: (i16, i16)) -> impl Iterator<Item = (i16, i16)> {
    bresenham::Bresenham::new(
        (from.0 as isize, from.1 as isize),
        (to.0 as isize, to.1 as isize),
    )
    .skip(1) // skip 'self'
    .map(|p| (p.0 as i16, p.1 as i16))
    .chain(std::iter::once(to))
}

/// Y is vertical
#[derive(Component, Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Pos(pub i16, pub i16, pub i16);

impl Pos {
    pub const fn in_bounds(&self) -> bool {
        0 <= self.0
            && self.0 < SIZE.0
            && 0 <= self.1
            && self.1 < SIZE.1
            && 0 <= self.2
            && self.2 < SIZE.2
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

    pub fn nbor(self, offset: Self) -> Option<Self> {
        let nbor = Self(self.0 + offset.0, self.1 + offset.1, self.2 + offset.2);
        if nbor.in_bounds() {
            assert!(
                self.is_potential_nbor(nbor) || nbor == self,
                "{:?} (= {:?} + {:?}) is not a nbor",
                nbor,
                self,
                offset
            );
            Some(nbor)
        } else {
            None
        }
    }

    pub fn vec3(self) -> Vec3 {
        Vec3::new(
            f32::from(self.0) * ADJACENT.f32(),
            f32::from(self.1) * VERTICAL.f32(),
            f32::from(self.2) * ADJACENT.f32(),
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

pub const SIZE: Pos = Pos(20 * 24, 10, 20 * 24);

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
