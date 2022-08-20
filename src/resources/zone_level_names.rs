use crate::prelude::*;
use bevy::utils::HashMap;

pub(crate) struct ZoneLevelNames {
    world_path: WorldPath,
    names: HashMap<ZoneLevel, ObjectName>,
}

impl ZoneLevelNames {
    pub(crate) fn new(world_path: WorldPath) -> Self {
        Self {
            world_path,
            names: HashMap::default(),
        }
    }

    pub(crate) fn get(&mut self, zone_level: ZoneLevel) -> Option<&ObjectName> {
        if !self.names.contains_key(&zone_level) {
            let overzone = Overzone::from(Zone::from(zone_level));
            let overmap_path = OvermapPath::new(&self.world_path, overzone);
            if let Ok(overmap) = Overmap::try_from(&overmap_path) {
                for level in Level::ALL {
                    self.names.extend(
                        overmap.layers[level.index()]
                            .0
                            .load_as_overzone(overzone, level)
                            .iter()
                            .map(|(k, v)| (*k, (*v).clone())),
                    );
                }
            }
        }
        self.names.get(&zone_level)
    }
}
