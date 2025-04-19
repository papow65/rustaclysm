use crate::{Exploration, Focus, Level, Overzone, Pos, Region, SubzoneLevel, Zone, ZoneLevel};
use bevy::platform::collections::{HashMap, HashSet, hash_map::Entry};
use bevy::prelude::{Resource, Visibility};

/// Ever seen by the player character
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum SeenFrom {
    FarAway,
    CloseBy,
    Never,
}

#[derive(Default, Resource)]
pub(crate) struct Explored {
    zone_levels: HashMap<ZoneLevel, SeenFrom>,
    pos: HashMap<Pos, bool>,
    loaded_overzones: HashSet<Overzone>,
}

impl Explored {
    fn mark_pos_seen(&mut self, pos: Pos) {
        let zone_level = ZoneLevel {
            zone: Zone::from(pos),
            level: pos.level.min(Level::ZERO), // Lower the zone level to the ground
        };
        self.zone_levels.insert(zone_level, SeenFrom::CloseBy);

        self.pos.insert(pos, true);
    }

    fn add_subzone_level(&mut self, subzone_level: SubzoneLevel, pos: &[Pos]) {
        let zone_level = ZoneLevel {
            zone: ZoneLevel::from(subzone_level).zone,
            level: subzone_level.level.min(Level::ZERO), // Lower the zone level to the ground
        };
        self.zone_levels.insert(zone_level, SeenFrom::CloseBy);

        for &pos in pos {
            self.pos.insert(pos, true);
        }
    }

    fn add_overzone(&mut self, overzone: Overzone, zone_levels: &[ZoneLevel]) {
        self.loaded_overzones.insert(overzone);

        for &zone_level in zone_levels {
            if let Entry::Vacant(vacant) = self.zone_levels.entry(zone_level) {
                vacant.insert(SeenFrom::FarAway);
            }
        }
    }

    pub(crate) fn add<'e>(&mut self, explorations: impl Iterator<Item = &'e Exploration>) {
        for exploration in explorations {
            match exploration {
                Exploration::Pos(pos) => self.mark_pos_seen(*pos),
                Exploration::SubzoneLevel(subzone_level, pos) => {
                    self.add_subzone_level(*subzone_level, pos);
                }
                Exploration::Overzone(overzone, zone_levels) => {
                    self.add_overzone(*overzone, zone_levels);
                }
            }
        }
    }

    pub(crate) fn has_zone_level_been_seen(&self, zone_level: ZoneLevel) -> Option<SeenFrom> {
        self.zone_levels.get(&zone_level).copied().or_else(|| {
            self.loaded_overzones
                .contains(&Overzone::from(zone_level.zone))
                .then_some(SeenFrom::Never)
        })
    }

    pub(crate) fn has_pos_been_seen(&self, pos: Pos) -> bool {
        self.pos.get(&pos) == Some(&true)
    }

    pub(crate) fn loaded(&self) -> bool {
        !self.zone_levels.is_empty()
    }

    pub(crate) fn zone_level_visibility(
        &self,
        focus: &Focus,
        zone_level: ZoneLevel,
        visible_region: &Region,
    ) -> Visibility {
        if zone_level.level == Level::from(focus).min(Level::ZERO)
            && zone_level.subzone_levels().iter().all(|subzone_level| {
                visible_region.contains_zone_level(ZoneLevel::from(*subzone_level))
                    && self
                        .has_zone_level_been_seen(zone_level)
                        .is_some_and(|seen_from| seen_from != SeenFrom::Never)
            })
        {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    }
}
