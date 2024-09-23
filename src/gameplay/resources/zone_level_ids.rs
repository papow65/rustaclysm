use crate::gameplay::{AssetState, Level, OvermapManager, Overzone, RepetitionBlockExt, ZoneLevel};
use bevy::{prelude::Resource, utils::HashMap};
use cdda::{FlatVec, ObjectId, Overmap, OvermapLevel};

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
                    fallback = Self::fallback_overmap();
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

    pub(crate) fn fallback_overmap() -> Overmap {
        Overmap {
            layers: [
                OvermapLevel::all(ObjectId::new("deep_rock")),
                OvermapLevel::all(ObjectId::new("deep_rock")),
                OvermapLevel::all(ObjectId::new("deep_rock")),
                OvermapLevel::all(ObjectId::new("deep_rock")),
                OvermapLevel::all(ObjectId::new("deep_rock")),
                OvermapLevel::all(ObjectId::new("deep_rock")),
                OvermapLevel::all(ObjectId::new("deep_rock")),
                OvermapLevel::all(ObjectId::new("empty_rock")),
                OvermapLevel::all(ObjectId::new("empty_rock")),
                OvermapLevel::all(ObjectId::new("solid_earth")),
                OvermapLevel::all(ObjectId::new("field")),
                OvermapLevel::all(ObjectId::new("open_air")),
                OvermapLevel::all(ObjectId::new("open_air")),
                OvermapLevel::all(ObjectId::new("open_air")),
                OvermapLevel::all(ObjectId::new("open_air")),
                OvermapLevel::all(ObjectId::new("open_air")),
                OvermapLevel::all(ObjectId::new("open_air")),
                OvermapLevel::all(ObjectId::new("open_air")),
                OvermapLevel::all(ObjectId::new("open_air")),
                OvermapLevel::all(ObjectId::new("open_air")),
                OvermapLevel::all(ObjectId::new("open_air")),
            ],
            region_id: serde_json::Value::Null,
            monster_groups: serde_json::Value::Null,
            cities: serde_json::Value::Null,
            connections_out: serde_json::Value::Null,
            radios: serde_json::Value::Null,
            monster_map: FlatVec(Vec::new()),
            tracked_vehicles: serde_json::Value::Null,
            scent_traces: serde_json::Value::Null,
            npcs: serde_json::Value::Null,
            camps: serde_json::Value::Null,
            overmap_special_placements: serde_json::Value::Null,
            mapgen_arg_storage: None,
            mapgen_arg_index: None,
            joins_used: None,
            predecessors: None,
        }
    }
}
