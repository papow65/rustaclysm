use crate::prelude::*;
use bevy::utils::HashMap;

pub(crate) struct Explored {
    sav_path: SavPath,
    explored: HashMap<ZoneLevel, bool>,
}

impl Explored {
    pub(crate) fn new(sav_path: SavPath) -> Self {
        Self {
            sav_path,
            explored: HashMap::default(),
        }
    }

    pub(crate) fn mark_seen(&mut self, zone_level: ZoneLevel) {
        self.explored.insert(zone_level, true);
    }

    pub(crate) fn has_been_seen(&mut self, zone_level: ZoneLevel) -> bool {
        if !self.explored.contains_key(&zone_level) {
            let overzone = Overzone::from(Zone::from(zone_level));
            let overmap_buffer_path = OvermapBufferPath::new(&self.sav_path, overzone);
            let buffer =
                OvermapBuffer::try_from(overmap_buffer_path).expect("Failed loading overzone");
            for level in Level::ALL {
                self.explored.extend(
                    buffer
                        .visible
                        .get(level.index())
                        .expect("level missing")
                        .load_as_overzone(overzone, level)
                        .iter()
                        .map(|(k, v)| (*k, *(*v))),
                );
            }
        }

        /*println!(
            "Explored.is_explored({zone_level:?}) {} -> {:?}",
            self.explored.len(),
            self.explored.get(&zone_level)
        );*/
        *self.explored.get(&zone_level).unwrap()
    }
}
