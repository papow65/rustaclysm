use crate::gameplay::{Level, OvermapAsset, Overzone, RepetitionBlockExt as _, ZoneLevel};
use bevy::{prelude::Resource, utils::HashMap};
use cdda_json_files::{FlatVec, ObjectId, Overmap, OvermapLevel};

#[derive(Default, Resource)]
pub(crate) struct ZoneLevelIds {
    names: HashMap<ZoneLevel, ObjectId>,
    loaded_overzones: Vec<Overzone>,
}

impl ZoneLevelIds {
    pub(crate) fn get(&self, zone_level: ZoneLevel) -> Option<&ObjectId> {
        self.names.get(&zone_level)
    }

    pub(crate) fn load(&mut self, overzone: Overzone, overmap: &OvermapAsset) {
        if !self.loaded_overzones.contains(&overzone) {
            for level in Level::ALL {
                self.names.extend(
                    overmap.0.layers[level.index()]
                        .0
                        .load_as_overzone(overzone, level)
                        .into_iter()
                        .map(|(k, v)| (k, v.clone())),
                );
            }
            self.loaded_overzones.push(overzone);
        }
    }

    pub(crate) fn create_missing(&mut self, overzone: Overzone) {
        let fallback = OvermapAsset(Overmap {
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
        });

        self.load(overzone, &fallback);
    }
}
