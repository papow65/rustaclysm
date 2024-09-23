use bevy::prelude::Vec3;
use cdda::At;
use units::Distance;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LevelOffset {
    pub(crate) h: i8,
}

impl LevelOffset {
    pub(crate) const ZERO: Self = Self { h: 0 };
    pub(crate) const UP: Self = Self { h: 1 };
    pub(crate) const DOWN: Self = Self { h: -1 };

    pub(crate) fn f32(&self) -> f32 {
        f32::from(self.h) * Distance::VERTICAL.meter_f32()
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) struct PosOffset {
    pub(crate) x: i32,
    pub(crate) level: LevelOffset,
    pub(crate) z: i32,
}

impl PosOffset {
    pub(crate) const HERE: Self = Self {
        x: 0,
        level: LevelOffset::ZERO,
        z: 0,
    };

    pub(crate) fn player_hint(&self) -> &str {
        if self.x == 0 && self.z == 0 {
            match self.level {
                LevelOffset { h } if h > 0 => "U",
                LevelOffset::ZERO => "H",
                _ => "D",
            }
        } else if 2 * self.z.abs() <= self.x.abs() {
            if 0 < self.x {
                "E"
            } else {
                "W"
            }
        } else if 2 * self.x.abs() <= self.z.abs() {
            if 0 < self.z {
                "S"
            } else {
                "N"
            }
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

    pub(crate) const fn down(&self) -> Self {
        Self {
            level: LevelOffset {
                h: self.level.h - 1,
            },
            ..*self
        }
    }

    pub(crate) fn vec3(&self) -> Vec3 {
        Vec3::new(
            f64::from(self.x) as f32 * Distance::ADJACENT.meter_f32(),
            self.level.f32(),
            f64::from(self.z) as f32 * Distance::ADJACENT.meter_f32(),
        )
    }

    pub(crate) const fn get<'a, T>(&'a self, at: &'a At<T>) -> Option<&'a T> {
        if self.x as u8 == at.x && self.z as u8 == at.y {
            Some(&at.obj)
        } else {
            None
        }
    }
}
