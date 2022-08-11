use crate::prelude::*;
use bevy::utils::HashMap;

pub struct ZoneLevelNames(HashMap<ZoneLevel, ObjectName>);

impl ZoneLevelNames {
    pub fn new() -> Self {
        Self(HashMap::default())
    }

    pub fn get(&mut self, zone_level: ZoneLevel) -> Option<&ObjectName> {
        if !self.0.contains_key(&zone_level) {
            let overzone = Overzone::from(Zone::from(zone_level));
            if let Ok(overmap) = Overmap::try_from(overzone) {
                for level in Level::ALL {
                    let mut i: i32 = 0;
                    for (name, amount) in &overmap.layers[level.index()].0 {
                        let amount = i32::from(*amount);
                        for j in i..i + amount {
                            let x = overzone.base_zone().x + j.rem_euclid(180);
                            let z = overzone.base_zone().z + j.div_euclid(180);
                            self.0.insert(ZoneLevel { x, level, z }, name.clone());
                        }
                        i += amount;
                    }
                    assert!(i == 32400, "{i}");
                }
            }
        }
        self.0.get(&zone_level)
    }
}
