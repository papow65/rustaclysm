use crate::prelude::{CddaAmount, Level, ObjectId, Overzone, PathFor, RepetitionBlock, WorldPath};
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
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Overmap {
    pub(crate) layers: [OvermapLevel; Level::AMOUNT],

    #[allow(unused)] // TODO
    region_id: serde_json::Value,

    #[allow(unused)] // TODO
    monster_groups: serde_json::Value,

    #[allow(unused)] // TODO
    cities: serde_json::Value,

    #[allow(unused)] // TODO
    connections_out: serde_json::Value,

    #[allow(unused)] // TODO
    radios: serde_json::Value,

    #[allow(unused)] // TODO
    monster_map: serde_json::Value,

    #[allow(unused)] // TODO
    tracked_vehicles: serde_json::Value,

    #[allow(unused)] // TODO
    scent_traces: serde_json::Value,

    #[allow(unused)] // TODO
    npcs: serde_json::Value,

    #[allow(unused)] // TODO
    camps: serde_json::Value,

    #[allow(unused)] // TODO
    overmap_special_placements: serde_json::Value,

    #[allow(unused)] // TODO
    mapgen_arg_storage: Option<serde_json::Value>,

    #[allow(unused)] // TODO
    mapgen_arg_index: Option<serde_json::Value>,

    #[allow(unused)] // TODO
    joins_used: Option<serde_json::Value>,

    #[allow(unused)] // TODO
    predecessors: Option<serde_json::Value>,
}

impl Overmap {
    pub(crate) fn fallback() -> Self {
        Self {
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
            monster_map: serde_json::Value::Null,
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

impl TryFrom<&OvermapPath> for Overmap {
    type Error = ();
    fn try_from(overmap_path: &OvermapPath) -> Result<Self, ()> {
        //println!("Path: {filepath}");
        read_to_string(&overmap_path.0)
            .ok()
            .map(|s| {
                let first_newline = s.find('\n').unwrap();
                let after_first_line = s.split_at(first_newline).1;
                serde_json::from_str(after_first_line).unwrap_or_else(|err| panic!("{err:?}"))
            })
            .ok_or(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OvermapLevel(pub(crate) RepetitionBlock<ObjectId>);

impl OvermapLevel {
    fn all(id: ObjectId) -> Self {
        Self(RepetitionBlock::new(CddaAmount {
            obj: id,
            amount: 180 * 180,
        }))
    }
}
