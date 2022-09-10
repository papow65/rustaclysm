use crate::prelude::{Level, ObjectName, Overzone, PathFor, RepetitionBlock, WorldPath};
use serde::Deserialize;
use std::fs::read_to_string;

pub(crate) type OvermapPath = PathFor<Overmap>;

impl OvermapPath {
    pub(crate) fn new(world_path: &WorldPath, overzone: Overzone) -> Self {
        Self::init(
            world_path
                .0
                .join(format!("o.{}.{}", overzone.x, overzone.z)),
        )
    }
}

/** Corresponds to an 'overmap' in CDDA. It defines the layout of 180x180 `Zone`s. */
#[allow(unused)]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Overmap {
    pub(crate) layers: [OvermapLevel; Level::AMOUNT],
    region_id: serde_json::Value,
    monster_groups: serde_json::Value,
    cities: serde_json::Value,
    connections_out: serde_json::Value,
    radios: serde_json::Value,
    monster_map: serde_json::Value,
    tracked_vehicles: serde_json::Value,
    scent_traces: serde_json::Value,
    npcs: serde_json::Value,
    camps: serde_json::Value,
    overmap_special_placements: serde_json::Value,
    mapgen_arg_storage: serde_json::Value,
    mapgen_arg_index: serde_json::Value,
    joins_used: Option<serde_json::Value>,
    predecessors: Option<serde_json::Value>,
}

impl TryFrom<&OvermapPath> for Overmap {
    type Error = ();
    fn try_from(overmap_path: &OvermapPath) -> Result<Self, ()> {
        //println!("Path: {filepath}");
        read_to_string(&overmap_path.0)
            .ok()
            .map(|s| s.split_at(s.find('\n').unwrap()).1.to_string())
            .map(|s| serde_json::from_str(s.as_str()).unwrap())
            .ok_or(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OvermapLevel(pub(crate) RepetitionBlock<ObjectName>);
