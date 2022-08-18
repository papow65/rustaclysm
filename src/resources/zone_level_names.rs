use crate::prelude::*;
use bevy::utils::HashMap;

#[derive(Default)]
pub(crate) struct ZoneLevelNames(HashMap<ZoneLevel, ObjectName>);

impl ZoneLevelNames {
    pub(crate) fn get(&mut self, zone_level: ZoneLevel) -> Option<&ObjectName> {
        if !self.0.contains_key(&zone_level) {
            let overzone = Overzone::from(Zone::from(zone_level));
            if let Ok(overmap) = Overmap::try_from(overzone) {
                for level in Level::ALL {
                    self.0.extend(
                        overmap.layers[level.index()]
                            .0
                            .load_as_overzone(overzone, level)
                            .iter()
                            .map(|(k, v)| (*k, (*v).clone())),
                    );
                }
            }
        }
        self.0.get(&zone_level)
    }
}
