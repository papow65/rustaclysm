use crate::prelude::{
    At, AtVec, FieldVec, ObjectName, PathFor, Repetition, RepetitionBlock, WorldPath, ZoneLevel,
};
use bevy::utils::HashMap;
use serde::Deserialize;
use std::fs::read_to_string;

pub(crate) type MapPath = PathFor<Map>;

impl MapPath {
    pub(crate) fn new(world_path: &WorldPath, zone_level: ZoneLevel) -> Self {
        Self::init(
            world_path
                .0
                .join("maps")
                .join(format!(
                    "{}.{}.{}",
                    zone_level.x.div_euclid(32),
                    zone_level.z.div_euclid(32),
                    zone_level.level.h,
                ))
                .join(format!(
                    "{}.{}.{}.map",
                    zone_level.x, zone_level.z, zone_level.level.h
                )),
        )
    }
}

// Reference: https://github.com/CleverRaven/Cataclysm-DDA/blob/master/src/savegame_json.cpp

/** Corresponds to a 'map' in CDDA. It defines the layout of a `ZoneLevel`. */
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Map(pub(crate) Vec<Submap>);

impl TryFrom<MapPath> for Option<Map> {
    type Error = serde_json::Error;
    fn try_from(map_path: MapPath) -> Result<Option<Map>, Self::Error> {
        read_to_string(&map_path.0)
            .ok()
            .map(|s| {
                println!("Found map: {}", map_path.0.display());
                s
            })
            .map_or(std::result::Result::Ok(Option::None), |s| {
                serde_json::from_str::<Map>(s.as_str()).map(Some)
            })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Submap {
    #[allow(unused)]
    version: u64,

    pub(crate) coordinates: (i32, i32, i32),

    #[allow(unused)]
    turn_last_touched: u64,

    #[allow(unused)]
    temperature: i64,

    #[allow(unused)]
    radiation: Vec<i64>,

    pub(crate) terrain: RepetitionBlock<ObjectName>,
    pub(crate) furniture: Vec<At<ObjectName>>,
    pub(crate) items: AtVec<Vec<Repetition<CddaItem>>>,

    #[allow(unused)]
    traps: AtVec<ObjectName>,

    pub(crate) fields: AtVec<FieldVec>,

    #[allow(unused)]
    cosmetics: Vec<(u8, u8, String, String)>,

    pub(crate) spawns: Vec<Spawn>,

    #[allow(unused)]
    vehicles: Vec<serde_json::Value>, // grep -orIE 'vehicles":\[[^]]+.{80}'  assets/save/maps/ | less

    #[allow(unused)]
    partial_constructions: Vec<serde_json::Value>,

    #[allow(unused)]
    computers: Option<Vec<serde_json::Value>>,
}

#[derive(Debug)]
pub(crate) struct Furniture {
    #[allow(unused)]
    tile_name: ObjectName,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CddaItem {
    pub(crate) typeid: ObjectName,
    snip_id: Option<String>,
    pub(crate) charges: Option<u32>,
    active: Option<bool>,
    corpse: Option<String>,
    name: Option<String>,
    owner: Option<String>,
    bday: Option<i64>,
    last_temp_check: Option<u64>,
    specific_energy: Option<Number>,
    temperature: Option<Number>,
    item_vars: Option<HashMap<String, String>>,
    item_tags: Option<Vec<String>>,
    contents: Option<CddaContainer>,
    components: Option<Vec<CddaItem>>,
    is_favorite: Option<bool>,
    relic_data: Option<serde_json::Value>,
    damaged: Option<i64>,
    current_phase: Option<u8>,
    faults: Option<Vec<String>>,
    rot: Option<i64>,
    curammo: Option<String>,
    item_counter: Option<u8>,
    variant: Option<String>,
    recipe_charges: Option<u8>,
    poison: Option<u8>,
    burnt: Option<serde_json::Value>,
    craft_data: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CddaContainer {
    #[allow(unused)]
    contents: Vec<Pocket>,

    #[allow(unused)]
    additional_pockets: Option<Vec<Pocket>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Pocket {
    #[allow(unused)]
    pocket_type: u8,

    #[allow(unused)]
    contents: Vec<CddaItem>,

    #[allow(unused)]
    _sealed: bool,

    #[allow(unused)]
    allowed: Option<bool>,

    #[allow(unused)]
    favorite_settings: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Spawn {
    pub(crate) spawn_type: ObjectName,

    #[allow(unused)]
    count: i32,

    pub(crate) x: i32,
    pub(crate) z: i32,

    #[allow(unused)]
    faction_id: i32,

    #[allow(unused)]
    mission_id: i32,

    #[allow(unused)]
    friendly: bool,

    #[allow(unused)]
    name: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum Number {
    Int(i64),
    Text(String),
}
