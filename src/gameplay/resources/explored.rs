use crate::prelude::*;
use bevy::{ecs::system::Resource, utils::HashMap};

/** Ever seen by the player character */
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SeenFrom {
    FarAway,
    CloseBy,
    Never,
}

#[derive(Resource)]
pub(crate) struct Explored {
    sav_path: SavPath,
    zone_level: HashMap<ZoneLevel, SeenFrom>,
    pos: HashMap<Pos, bool>,
}

impl Explored {
    pub(crate) fn new(sav_path: SavPath) -> Self {
        Self {
            sav_path,
            zone_level: HashMap::default(),
            pos: HashMap::default(),
        }
    }

    fn load_if_missing(&mut self, zone_level: ZoneLevel) {
        if !self.zone_level.contains_key(&zone_level) {
            let overzone = Overzone::from(zone_level.zone);
            let overmap_buffer_path = OvermapBufferPath::new(&self.sav_path, overzone);
            let buffer = OvermapBuffer::try_from(overmap_buffer_path)
                .ok()
                .unwrap_or_else(OvermapBuffer::fallback);
            for level in Level::ALL {
                self.zone_level.extend(
                    buffer
                        .visible
                        .get(level.index())
                        .expect("level missing")
                        .load_as_overzone(overzone, level)
                        .iter()
                        .map(|(k, v)| {
                            (
                                *k,
                                if **v {
                                    SeenFrom::FarAway
                                } else {
                                    SeenFrom::Never
                                },
                            )
                        }),
                );
            }
        }
    }

    pub(crate) fn mark_pos_seen(&mut self, pos: Pos) {
        // Lower the zone level to the ground
        let zone_level = ZoneLevel {
            zone: Zone::from(pos),
            level: pos.level.min(Level::ZERO),
        };

        // Make sure the zone_level will not be overwritten later by loading a nearby zone_level.
        self.load_if_missing(zone_level);

        self.zone_level.insert(zone_level, SeenFrom::CloseBy);
        self.pos.insert(pos, true);
    }

    pub(crate) fn has_zone_level_been_seen(&mut self, zone_level: ZoneLevel) -> SeenFrom {
        self.load_if_missing(zone_level);
        self.zone_level.get(&zone_level).unwrap().clone()
    }

    pub(crate) fn has_pos_been_seen(&self, pos: Pos) -> bool {
        self.pos.get(&pos) == Some(&true)
    }
}
