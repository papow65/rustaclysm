use crate::prelude::{Level, Zone, ZoneLevel};

#[derive(Clone, Debug)]
pub(crate) struct ZoneRegion {
    from: Zone,
    /** including */
    to: Zone,
}

impl ZoneRegion {
    /** Not well-formed! */
    const MIN: Self = Self::new(Zone::MAX, Zone::MIN);
    const MAX: Self = Self::new(Zone::MIN, Zone::MAX);

    pub(crate) const fn new(from: Zone, to: Zone) -> Self {
        Self { from, to }
    }

    pub(crate) fn well_formed(&self) -> bool {
        self.from.x <= self.to.x && self.from.z <= self.to.z
    }

    pub(crate) fn contains_zone(&self, zone: Zone) -> bool {
        (self.from.x..=self.to.x).contains(&zone.x) && (self.from.z..=self.to.z).contains(&zone.z)
    }

    pub(crate) fn contains_zone_region(&self, other: &Self) -> bool {
        self.from.x <= other.from.x
            && self.from.z <= other.from.z
            && other.to.x <= self.to.x
            && other.to.z <= self.to.z
    }
}

#[derive(Clone, Default, Debug)]
pub(crate) struct Region {
    zone_regions: [Option<ZoneRegion>; Level::AMOUNT],
}

impl Region {
    pub(crate) fn new(zone_levels: &[ZoneLevel]) -> Self {
        Self {
            zone_regions: Level::ALL.map(|level| {
                let zone_levels = zone_levels
                    .iter()
                    .filter(|zone_level| zone_level.level == level)
                    .copied()
                    .collect::<Vec<ZoneLevel>>();
                if zone_levels.is_empty() {
                    return None;
                }
                let xs = zone_levels
                    .iter()
                    .map(|zone_level| zone_level.x)
                    .collect::<Vec<i32>>();
                let zs = zone_levels
                    .iter()
                    .map(|zone_level| zone_level.z)
                    .collect::<Vec<i32>>();

                Some(ZoneRegion::new(
                    Zone {
                        x: *xs.iter().min().unwrap(),
                        z: *zs.iter().min().unwrap(),
                    },
                    Zone {
                        x: *xs.iter().max().unwrap(),
                        z: *zs.iter().max().unwrap(),
                    },
                ))
            }),
        }
    }

    pub(crate) fn clamp(&self, inner: &Region, outer: &Region) -> Self {
        let mut i = 0;
        Self {
            zone_regions: self.zone_regions.clone().map(move |region_level| {
                let zone_region = if let Some(zone_region) = region_level {
                    let inner = inner.zone_regions[i].clone().unwrap_or(ZoneRegion::MIN);
                    let outer = outer.zone_regions[i].clone().unwrap_or(ZoneRegion::MAX);
                    assert!(
                        outer.contains_zone_region(&inner),
                        "Level {:?}: {:?} does not contain {:?}",
                        Level::ALL[i],
                        &outer,
                        &inner
                    );

                    Some(ZoneRegion::new(
                        Zone {
                            x: zone_region.from.x.clamp(outer.from.x, inner.from.x),
                            z: zone_region.from.z.clamp(outer.from.z, inner.from.z),
                        },
                        Zone {
                            x: zone_region.to.x.clamp(inner.to.x, outer.to.x),
                            z: zone_region.to.z.clamp(inner.to.z, outer.to.z),
                        },
                    ))
                } else {
                    inner.zone_regions[i].clone()
                };
                i += 1; // for next level
                zone_region
            }),
        }
    }

    pub(crate) fn contains_zone_level(&self, zone_level: ZoneLevel) -> bool {
        if let Some(zone_region) = &self.zone_regions[zone_level.level.index()] {
            zone_region.contains_zone(Zone::from(zone_level))
        } else {
            false
        }
    }

    pub(crate) fn zone_levels(&self) -> Vec<ZoneLevel> {
        let mut zone_levels = Vec::new();
        for level in Level::ALL {
            if let Some(zone_region) = &self.zone_regions[level.index()] {
                for x in zone_region.from.x..=zone_region.to.x {
                    for z in zone_region.from.z..=zone_region.to.z {
                        zone_levels.push(ZoneLevel { x, level, z });
                    }
                }
            }
        }
        zone_levels
    }
}

impl From<&ZoneRegion> for Region {
    fn from(zone_region: &ZoneRegion) -> Self {
        Self {
            zone_regions: Level::ALL.map(|_| Some(zone_region.clone())),
        }
    }
}
