use crate::prelude::{Level, SubzoneLevel};
use std::ops::RangeInclusive;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SubzoneRegion {
    x: RangeInclusive<i32>,
    z: RangeInclusive<i32>,
}

impl SubzoneRegion {
    pub(crate) const MAX: Self = Self {
        x: i32::MIN..=i32::MAX,
        z: i32::MIN..=i32::MAX,
    };

    pub(crate) const fn new(x: RangeInclusive<i32>, z: RangeInclusive<i32>) -> Self {
        Self { x, z }
    }

    pub(crate) fn contains_subzone(&self, subzone_level: SubzoneLevel) -> bool {
        self.x.contains(&subzone_level.x) && self.z.contains(&subzone_level.z)
    }

    pub(crate) fn contains_subzone_region(&self, other: &Self) -> bool {
        self.x.contains(other.x.start())
            && self.x.contains(other.x.end())
            && self.z.contains(other.z.start())
            && self.z.contains(other.z.end())
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub(crate) struct Region {
    subzone_regions: [Option<SubzoneRegion>; Level::AMOUNT],
}

impl Region {
    pub(crate) fn new(subzone_levels: &[SubzoneLevel]) -> Self {
        Self {
            subzone_regions: Level::ALL.map(|level| {
                let subzone_levels = subzone_levels
                    .iter()
                    .filter(|subzone_level| subzone_level.level == level)
                    .copied()
                    .collect::<Vec<SubzoneLevel>>();
                if subzone_levels.is_empty() {
                    return None;
                }
                let xs = subzone_levels
                    .iter()
                    .map(|subzone_level| subzone_level.x)
                    .collect::<Vec<i32>>();
                let zs = subzone_levels
                    .iter()
                    .map(|subzone_level| subzone_level.z)
                    .collect::<Vec<i32>>();

                Some(SubzoneRegion::new(
                    *xs.iter().min().unwrap()..=*xs.iter().max().unwrap(),
                    *zs.iter().min().unwrap()..=*zs.iter().max().unwrap(),
                ))
            }),
        }
    }

    pub(crate) fn ground_only(&self) -> Self {
        let mut i = 0;
        Self {
            subzone_regions: self.subzone_regions.clone().map(|subzone_region| {
                let subzone_region = subzone_region.filter(|_| i <= Level::ZERO.index());
                i += 1; // for next level
                subzone_region
            }),
        }
    }

    pub(crate) fn clamp(&self, inner: &Self, outer: &Self) -> Self {
        let mut i = 0;
        Self {
            subzone_regions: self.subzone_regions.clone().map(move |region_level| {
                let subzone_region = if let Some(subzone_region) = region_level {
                    let inner = inner.subzone_regions[i]
                        .clone()
                        .unwrap_or_else(|| subzone_region.clone());
                    let outer = outer.subzone_regions[i]
                        .clone()
                        .unwrap_or(SubzoneRegion::MAX);
                    assert!(
                        outer.contains_subzone_region(&inner),
                        "Level {:?}: {:?} does not contain {:?}",
                        Level::ALL[i],
                        &outer,
                        &inner
                    );

                    let x_from = *subzone_region
                        .x
                        .start()
                        .clamp(outer.x.start(), inner.x.start());
                    let x_to = *subzone_region.x.end().clamp(inner.x.end(), outer.x.end());
                    let z_from = *subzone_region
                        .z
                        .start()
                        .clamp(outer.z.start(), inner.z.start());
                    let z_to = *subzone_region.z.end().clamp(inner.z.end(), outer.z.end());
                    Some(SubzoneRegion::new(x_from..=x_to, z_from..=z_to))
                } else {
                    inner.subzone_regions[i].clone()
                };
                i += 1; // for next level
                subzone_region
            }),
        }
    }

    pub(crate) fn contains_subzone_level(&self, subzone_level: SubzoneLevel) -> bool {
        if let Some(subzone_region) = &self.subzone_regions[subzone_level.level.index()] {
            subzone_region.contains_subzone(subzone_level)
        } else {
            false
        }
    }

    pub(crate) fn subzone_levels(&self) -> Vec<SubzoneLevel> {
        let mut subzone_levels = Vec::new();
        for level in Level::ALL {
            if let Some(subzone_region) = &self.subzone_regions[level.index()] {
                for x in subzone_region.clone().x {
                    for z in subzone_region.clone().z {
                        subzone_levels.push(SubzoneLevel { x, level, z });
                    }
                }
            }
        }
        subzone_levels
    }
}

impl From<&SubzoneRegion> for Region {
    fn from(subzone_region: &SubzoneRegion) -> Self {
        Self {
            subzone_regions: Level::ALL.map(|_| Some(subzone_region.clone())),
        }
    }
}
