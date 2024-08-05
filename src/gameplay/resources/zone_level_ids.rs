use crate::cdda::Overmap;
use crate::gameplay::{AssetState, Level, ObjectId, OvermapManager, Overzone, ZoneLevel};
use bevy::{prelude::Resource, utils::HashMap};

#[derive(Default, Resource)]
pub(crate) struct ZoneLevelIds {
    names: HashMap<ZoneLevel, ObjectId>,
    loaded_overzones: Vec<Overzone>,
}

impl ZoneLevelIds {
    pub(crate) fn get(
        &mut self,
        overmap_manager: &mut OvermapManager,
        zone_level: ZoneLevel,
    ) -> Option<&ObjectId> {
        if !self.names.contains_key(&zone_level) {
            let overzone = Overzone::from(zone_level.zone);
            let fallback;
            let overmap = match overmap_manager.get(overzone) {
                AssetState::Available { asset: overmap } => overmap,
                AssetState::Loading => {
                    return None;
                }
                AssetState::Nonexistent => {
                    fallback = Overmap::fallback();
                    &fallback
                }
            };
            self.load(overzone, overmap);
        }
        Some(
            self.names
                .get(&zone_level)
                .expect("zone level should be known"),
        )
    }

    pub(crate) fn load(&mut self, overzone: Overzone, overmap: &Overmap) {
        if !self.loaded_overzones.contains(&overzone) {
            for level in Level::ALL {
                self.names.extend(
                    overmap.layers[level.index()]
                        .0
                        .load_as_overzone(overzone, level)
                        .into_iter()
                        .map(|(k, v)| (k, v.clone())),
                );
            }
            self.loaded_overzones.push(overzone);
        }
    }
}
