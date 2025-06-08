use crate::{Level, Pos};
use bevy::prelude::Component;
use cdda_json_files::SubzoneOffset;
use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Component)]
#[component(immutable)]
pub struct SubzoneLevel {
    pub x: i32,
    pub level: Level,
    pub z: i32,
}

impl SubzoneLevel {
    pub const SIZE: i32 = 12;

    /// CDDA index in map
    #[must_use]
    pub const fn index(&self) -> usize {
        2 * (self.x as usize % 2) + (self.z as usize % 2)
    }

    /// CDDA coordinates
    #[must_use]
    pub const fn coordinates(&self) -> (i32, i32, i8) {
        (self.x, self.z, self.level.h)
    }

    #[must_use]
    pub const fn base_corner(&self) -> Pos {
        Pos::new(Self::SIZE * self.x, self.level, Self::SIZE * self.z)
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

impl From<SubzoneLevel> for SubzoneOffset {
    fn from(subzone_level: SubzoneLevel) -> Self {
        Self(
            subzone_level.x.div_euclid(180) as u16,
            subzone_level.z.div_euclid(180) as u16,
            subzone_level.level.h,
        )
    }
}
