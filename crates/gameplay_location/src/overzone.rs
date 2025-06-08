use crate::Zone;
use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Overzone {
    pub x: i32,
    pub z: i32,
}

impl Overzone {
    #[must_use]
    pub const fn base_zone(self) -> Zone {
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
