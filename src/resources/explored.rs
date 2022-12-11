use crate::prelude::*;
use bevy::{ecs::system::Resource, utils::HashMap};

#[derive(Clone, PartialEq)]
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

    pub(crate) fn _mark_zone_level_seen(&mut self, zone_level: ZoneLevel) {
        self.zone_level
            .entry(zone_level)
            .or_insert(SeenFrom::FarAway);
    }

    pub(crate) fn mark_pos_seen(&mut self, pos: Pos) {
        self.zone_level
            .insert(ZoneLevel::from(pos), SeenFrom::CloseBy);
        self.pos.insert(pos, true);
    }

    pub(crate) fn has_zone_level_been_seen(&mut self, zone_level: ZoneLevel) -> SeenFrom {
        if !self.zone_level.contains_key(&zone_level) {
            let overzone = Overzone::from(Zone::from(zone_level));
            let overmap_buffer_path = OvermapBufferPath::new(&self.sav_path, overzone);
            let buffer =
                OvermapBuffer::try_from(overmap_buffer_path).expect("Failed loading overzone");
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

        self.zone_level.get(&zone_level).unwrap().clone()
    }

    pub(crate) fn has_pos_been_seen(&self, pos: Pos) -> bool {
        self.pos.get(&pos) == Some(&true)
    }
}
