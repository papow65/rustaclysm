use crate::{Level, Pos, SubzoneLevel, ZoneLevel};
use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Zone {
    pub x: i32,
    pub z: i32,
}

impl Zone {
    pub const SIZE: i32 = 2 * SubzoneLevel::SIZE;

    #[must_use]
    pub const fn zone_level(self, level: Level) -> ZoneLevel {
        ZoneLevel { zone: self, level }
    }

    #[must_use]
    pub const fn offset(self, x: i32, z: i32) -> Self {
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
