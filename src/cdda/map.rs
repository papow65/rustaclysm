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

impl TryFrom<MapPath> for Map {
    type Error = ();
    fn try_from(map_path: MapPath) -> Result<Self, ()> {
        read_to_string(&map_path.0)
            .ok()
            .map(|s| {
                println!("Found map: {}", map_path.0.display());
                s
            })
            .map(|s| serde_json::from_str::<Self>(s.as_str()))
            .map(|r| r.map_err(|e| println!("{e}")).unwrap())
            .ok_or(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Submap {
    #[allow(unused)]
    pub(crate) version: u64,
    pub(crate) coordinates: (i32, i32, i32),

    #[allow(unused)]
    pub(crate) turn_last_touched: u64,

    #[allow(unused)]
    pub(crate) temperature: i64,

    #[allow(unused)]
    pub(crate) radiation: Vec<i64>,

    pub(crate) terrain: RepetitionBlock<ObjectName>,
    pub(crate) furniture: Vec<At<ObjectName>>,
    pub(crate) items: AtVec<Vec<Repetition<CddaItem>>>,

    #[allow(unused)]
    pub(crate) traps: AtVec<ObjectName>,

    pub(crate) fields: AtVec<FieldVec>,

    #[allow(unused)]
    pub(crate) cosmetics: Vec<(u8, u8, String, String)>,

    pub(crate) spawns: Vec<Spawn>,

    #[allow(unused)]
    pub(crate) vehicles: Vec<serde_json::Value>, // grep -orIE 'vehicles":\[[^]]+.{80}'  assets/save/maps/ | less

    #[allow(unused)]
    pub(crate) partial_constructions: Vec<serde_json::Value>,

    #[allow(unused)]
    pub(crate) computers: Option<Vec<serde_json::Value>>,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Furniture {
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
    specific_energy: Option<u64>,
    temperature: Option<u64>,
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

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CddaContainer {
    contents: Vec<Pocket>,
    additional_pockets: Option<Vec<Pocket>>,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Pocket {
    pocket_type: u8,
    contents: Vec<CddaItem>,
    _sealed: bool,
    allowed: Option<bool>,
    favorite_settings: Option<serde_json::Value>,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Spawn {
    pub(crate) spawn_type: ObjectName,
    count: i32,
    pub(crate) x: i32,
    pub(crate) z: i32,
    faction_id: i32,
    mission_id: i32,
    pub(crate) friendly: bool,
    pub(crate) name: Option<String>,
}
