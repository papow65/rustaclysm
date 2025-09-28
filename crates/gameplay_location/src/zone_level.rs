use crate::{Level, LevelOffset, Nbor, Pos, SubzoneLevel, Zone, ZoneLevelCache};
use bevy::prelude::Component;
use std::fmt;

// Manually deriving `Component`
#[derive(Copy, Clone, PartialEq, Eq, Hash, Component)]
#[component(immutable, on_insert=ZoneLevelCache::on_insert, on_replace=ZoneLevelCache::on_replace)]
pub struct ZoneLevel {
    pub zone: Zone,
    pub level: Level,
}

impl ZoneLevel {
    #[must_use]
    pub fn offset(self, offset: LevelOffset) -> Option<Self> {
        self.level.offset(offset).map(|level| Self {
            zone: self.zone,
            level,
        })
    }

    #[must_use]
    pub fn nbor(self, nbor: Nbor) -> Option<Self> {
        match nbor {
            Nbor::Up => self.level.up().map(|level| Self {
                zone: self.zone,
                level,
            }),
            Nbor::Down => self.level.down().map(|level| Self {
                zone: self.zone,
                level,
            }),
            Nbor::Horizontal(horizontal_direction) => {
                let (x, z) = horizontal_direction.offset();
                Some(Self {
                    zone: self.zone.offset(x, z),
                    level: self.level,
                })
            }
        }
    }

    #[must_use]
    pub const fn base_corner(&self) -> Pos {
        Pos::new(
            Zone::SIZE * self.zone.x,
            self.level,
            Zone::SIZE * self.zone.z,
        )
    }

    #[must_use]
    pub const fn center_pos(&self) -> Pos {
        self.base_corner().horizontal_offset(11, 11)
    }

    #[must_use]
    pub const fn subzone_levels(&self) -> [SubzoneLevel; 4] {
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
