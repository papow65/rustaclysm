use crate::prelude::*;
use bevy::{
    prelude::{AssetServer, Handle, Resource},
    utils::HashMap,
};

/** Ever seen by the player character */
#[derive(Copy, Clone, Debug, PartialEq)]
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
    loading: HashMap<Handle<OvermapBuffer>, Overzone>,
}

impl Explored {
    pub(crate) fn new(sav_path: SavPath) -> Self {
        Self {
            sav_path,
            zone_level: HashMap::default(),
            pos: HashMap::default(),
            loading: HashMap::default(),
        }
    }

    pub(crate) fn load(
        &mut self,
        handle: &Handle<OvermapBuffer>,
        overmap_buffer: &OvermapBuffer,
    ) -> Overzone {
        let overzone = self.loading.remove(handle).expect("Loading overmap buffer");
        for level in Level::GROUNDS {
            let overmap = overmap_buffer
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
                if 103 < zone_level.zone.x
                    && zone_level.zone.x < 109
                    && Level::new(-3) < zone_level.level
                    && 215 < zone_level.zone.z
                    && zone_level.zone.z < 221
                {
                    dbg!(zone_level);
                    dbg!(self.zone_level.get(&zone_level));
                }
            }
        }

        dbg!(overzone);
        dbg!(&self.loading);
        dbg!(self.zone_level.len());

        overzone
    }

    pub(crate) fn mark_pos_seen(&mut self, pos: Pos) {
        // Lower the zone level to the ground
        let zone_level = ZoneLevel {
            zone: Zone::from(pos),
            level: pos.level.min(Level::ZERO),
        };

        self.zone_level.insert(zone_level, SeenFrom::CloseBy);
        self.pos.insert(pos, true);
    }

    pub(crate) fn has_zone_level_been_seen(
        &mut self,
        asset_server: &AssetServer,
        zone_level: ZoneLevel,
    ) -> Option<SeenFrom> {
        self.zone_level.get(&zone_level).copied().or_else(|| {
            let overzone = Overzone::from(zone_level.zone);
            if !self.loading.values().any(|o| *o == overzone) {
                let handle = asset_server
                    .load::<OvermapBuffer, _>(OvermapBufferPath::new(&self.sav_path, overzone).0);
                self.loading.insert(handle, overzone);
                dbg!(&self.loading);
            }
            None
        })
    }

    pub(crate) fn has_pos_been_seen(&self, pos: Pos) -> bool {
        self.pos.get(&pos) == Some(&true)
    }
}
