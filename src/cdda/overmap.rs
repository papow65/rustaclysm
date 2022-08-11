use crate::prelude::{Level, ObjectName, Overzone};
use serde::Deserialize;
use std::fs::read_to_string;

/** Corresponds to an 'overmap' in CDDA. It defines the layout of 180x180 `Zone`s. */
#[allow(unused)]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Overmap {
    pub layers: [OvermapLevel; Level::AMOUNT],
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

impl TryFrom<Overzone> for Overmap {
    type Error = ();
    fn try_from(overzone: Overzone) -> Result<Self, ()> {
        let filepath = format!("assets/save/o.{}.{}", overzone.x, overzone.z);
        //println!("Path: {filepath}");
        read_to_string(&filepath)
            .ok()
            .map(|s| s.split_at(s.find('\n').unwrap()).1.to_string())
            .map(|s| serde_json::from_str(s.as_str()).unwrap())
            .ok_or(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OvermapLevel(pub Vec<(ObjectName, u16)>);
