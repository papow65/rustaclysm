use crate::gameplay::{Level, Overzone, Pos, Zone, ZoneLevel};
use bevy::prelude::Resource;
use bevy::utils::hashbrown::hash_map::Entry;
use bevy::utils::{HashMap, HashSet};

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
    pub(crate) fn mark_pos_seen(&mut self, pos: Pos) {
        // Lower the zone level to the ground
        let zone_level = ZoneLevel {
            zone: Zone::from(pos),
            level: pos.level.min(Level::ZERO),
        };

        self.zone_levels.insert(zone_level, SeenFrom::CloseBy);
        self.pos.insert(pos, true);
    }

    /// Also mark the overzone as loaded
    pub(crate) fn mark_zone_level_seen(&mut self, zone_level: ZoneLevel) {
        if let Entry::Vacant(vacant) = self.zone_levels.entry(zone_level) {
            vacant.insert(SeenFrom::FarAway);
            self.loaded_overzones
                .insert(Overzone::from(zone_level.zone));
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
}
