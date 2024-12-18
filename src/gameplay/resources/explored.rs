use crate::gameplay::cdda::RepetitionBlockExt as _;
use crate::gameplay::{Level, OvermapBufferAsset, Overzone, Pos, Zone, ZoneLevel};
use bevy::{prelude::Resource, utils::HashMap};

/// Ever seen by the player character
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum SeenFrom {
    FarAway,
    CloseBy,
    Never,
}

#[derive(Default, Resource)]
pub(crate) struct Explored {
    zone_level: HashMap<ZoneLevel, SeenFrom>,
    pos: HashMap<Pos, bool>,
    loaded_overzones: Vec<Overzone>,
}

impl Explored {
    pub(crate) fn mark_pos_seen(&mut self, pos: Pos) {
        // Lower the zone level to the ground
        let zone_level = ZoneLevel {
            zone: Zone::from(pos),
            level: pos.level.min(Level::ZERO),
        };

        self.zone_level.insert(zone_level, SeenFrom::CloseBy);
        self.pos.insert(pos, true);
    }

    pub(crate) fn has_zone_level_been_seen(&self, zone_level: ZoneLevel) -> Option<SeenFrom> {
        self.zone_level.get(&zone_level).copied()
    }

    pub(crate) fn has_pos_been_seen(&self, pos: Pos) -> bool {
        self.pos.get(&pos) == Some(&true)
    }

    pub(crate) fn load_buffer(&mut self, overzone: Overzone, overmap_buffer: &OvermapBufferAsset) {
        if !self.loaded_overzones.contains(&overzone) {
            for level in Level::GROUNDS {
                let overmap = overmap_buffer
                    .0
                    .visible
                    .get(level.index())
                    .expect("level missing")
                    .load_as_overzone(overzone, level);
                for (zone_level, seen) in overmap {
                    self.zone_level.entry(zone_level).or_insert(if *seen {
                        SeenFrom::FarAway
                    } else {
                        SeenFrom::Never
                    });
                }
            }
            self.loaded_overzones.push(overzone);
        }
    }

    pub(crate) fn loaded(&self) -> bool {
        !self.zone_level.is_empty()
    }
}
