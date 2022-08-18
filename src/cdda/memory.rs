use crate::prelude::*;
use bevy::utils::HashMap;

#[derive(Default)]
pub struct Memory {
    explored: HashMap<ZoneLevel, bool>,
}

impl Memory {
    pub fn has_been_seen(&mut self, zone_level: ZoneLevel) -> bool {
        if !self.explored.contains_key(&zone_level) {
            let overzone = Overzone::from(Zone::from(zone_level));
            let buffer = OvermapBuffer::try_from(overzone).unwrap();
            for level in Level::ALL {
                self.explored.extend(
                    buffer
                        .visible
                        .get(level.index())
                        .unwrap()
                        .load_as_overzone(overzone, level)
                        .iter()
                        .map(|(k, v)| (*k, *(*v))),
                );
            }
        }

        /*println!(
            "Memory.is_explored({zone_level:?}) {} -> {:?}",
            self.explored.len(),
            self.explored.get(&zone_level)
        );*/
        *self.explored.get(&zone_level).unwrap()
    }
}
