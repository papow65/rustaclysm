use crate::prelude::{Level, Zone, ZoneLevel};
use std::ops::RangeInclusive;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ZoneRegion {
    x: RangeInclusive<i32>,
    z: RangeInclusive<i32>,
}

impl ZoneRegion {
    pub(crate) const MAX: Self = Self {
        x: i32::MIN..=i32::MAX,
        z: i32::MIN..=i32::MAX,
    };

    pub(crate) const fn new(x: RangeInclusive<i32>, z: RangeInclusive<i32>) -> Self {
        Self { x, z }
    }

    pub(crate) fn contains_zone(&self, zone: Zone) -> bool {
        self.x.contains(&zone.x) && self.z.contains(&zone.z)
    }

    pub(crate) fn contains_zone_region(&self, other: &Self) -> bool {
        self.x.contains(other.x.start())
            && self.x.contains(other.x.end())
            && self.z.contains(other.z.start())
            && self.z.contains(other.z.end())
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
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
                    .map(|zone_level| zone_level.zone.x)
                    .collect::<Vec<i32>>();
                let zs = zone_levels
                    .iter()
                    .map(|zone_level| zone_level.zone.z)
                    .collect::<Vec<i32>>();

                Some(ZoneRegion::new(
                    *xs.iter().min().unwrap()..=*xs.iter().max().unwrap(),
                    *zs.iter().min().unwrap()..=*zs.iter().max().unwrap(),
                ))
            }),
        }
    }

    pub(crate) fn ground_only(&self) -> Self {
        let mut i = 0;
        Self {
            zone_regions: self.zone_regions.clone().map(|zone_region| {
                let zone_region = zone_region.filter(|_| i <= Level::ZERO.index());
                i += 1; // for next level
                zone_region
            }),
        }
    }

    pub(crate) fn clamp(&self, inner: &Self, outer: &Self) -> Self {
        let mut i = 0;
        Self {
            zone_regions: self.zone_regions.clone().map(move |region_level| {
                let zone_region = if let Some(zone_region) = region_level {
                    let inner = inner.zone_regions[i]
                        .clone()
                        .unwrap_or_else(|| zone_region.clone());
                    let outer = outer.zone_regions[i].clone().unwrap_or(ZoneRegion::MAX);
                    assert!(
                        outer.contains_zone_region(&inner),
                        "Level {:?}: {:?} does not contain {:?}",
                        Level::ALL[i],
                        &outer,
                        &inner
                    );

                    let x_from = *zone_region
                        .x
                        .start()
                        .clamp(outer.x.start(), inner.x.start());
                    let x_to = *zone_region.x.end().clamp(inner.x.end(), outer.x.end());
                    let z_from = *zone_region
                        .z
                        .start()
                        .clamp(outer.z.start(), inner.z.start());
                    let z_to = *zone_region.z.end().clamp(inner.z.end(), outer.z.end());
                    Some(ZoneRegion::new(x_from..=x_to, z_from..=z_to))
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
            zone_region.contains_zone(zone_level.zone)
        } else {
            false
        }
    }

    pub(crate) fn zone_levels(&self) -> Vec<ZoneLevel> {
        let mut zone_levels = Vec::new();
        for level in Level::ALL {
            if let Some(zone_region) = &self.zone_regions[level.index()] {
                for x in zone_region.clone().x {
                    for z in zone_region.clone().z {
                        zone_levels.push(Zone { x, z }.zone_level(level));
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
