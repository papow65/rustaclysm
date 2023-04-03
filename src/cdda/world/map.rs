use crate::prelude::{
    At, AtVec, CddaAmount, FieldVec, FlatVec, ObjectId, PathFor, Repetition, RepetitionBlock,
    SubzoneLevel, WorldPath, ZoneLevel,
};
use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::HashMap,
};
use serde::Deserialize;

pub(crate) type MapPath = PathFor<Map>;

impl MapPath {
    pub(crate) fn new(world_path: &WorldPath, zone_level: ZoneLevel) -> Self {
        Self::init(
            world_path
                .0
                .join("maps")
                .join(format!(
                    "{}.{}.{}",
                    zone_level.zone.x.div_euclid(32),
                    zone_level.zone.z.div_euclid(32),
                    zone_level.level.h,
                ))
                .join(format!(
                    "{}.{}.{}.map",
                    zone_level.zone.x, zone_level.zone.z, zone_level.level.h
                )),
        )
    }
}

// Reference: https://github.com/CleverRaven/Cataclysm-DDA/blob/master/src/savegame_json.cpp

/** Corresponds to a 'map' in CDDA. It defines the layout of a `ZoneLevel`. */
#[derive(Debug, Deserialize, TypeUuid)]
#[serde(deny_unknown_fields)]
#[uuid = "0ba81e33-a7c0-4366-a061-c37b2b0af4fa"]
pub(crate) struct Map(pub(crate) [Submap; 4]);

#[derive(Default)]
pub(crate) struct MapLoader;

impl AssetLoader for MapLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let map = serde_json::from_slice::<Map>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(map));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["map"]
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Submap {
    #[allow(unused)]
    version: u64,

    pub(crate) coordinates: (i32, i32, i8),

    #[allow(unused)]
    turn_last_touched: u64,

    #[allow(unused)]
    temperature: i64,

    #[allow(unused)]
    radiation: Vec<i64>,

    pub(crate) terrain: RepetitionBlock<ObjectId>,
    pub(crate) furniture: Vec<At<ObjectId>>,
    pub(crate) items: AtVec<Vec<Repetition<CddaItem>>>,

    #[allow(unused)]
    traps: Vec<At<ObjectId>>,

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

impl Submap {
    pub(crate) fn fallback(subzone_level: SubzoneLevel, zone_object_id: &ObjectId) -> Self {
        Submap {
            version: 0,
            turn_last_touched: 0,
            coordinates: subzone_level.coordinates(),
            temperature: 0,
            radiation: Vec::new(),
            terrain: RepetitionBlock::new(CddaAmount {
                obj: ObjectId::new(if zone_object_id == &ObjectId::new("open_air") {
                    "t_open_air"
                } else if zone_object_id == &ObjectId::new("solid_earth") {
                    "t_soil"
                } else if [ObjectId::new("empty_rock"), ObjectId::new("deep_rock")]
                    .contains(zone_object_id)
                {
                    "t_rock"
                } else if zone_object_id.is_moving_deep_water_zone() {
                    "t_water_moving_dp"
                } else if zone_object_id.is_still_deep_water_zone() {
                    "t_water_dp"
                } else if zone_object_id.is_grassy_zone() {
                    "t_grass"
                } else if zone_object_id.is_road_zone() {
                    "t_pavement"
                } else {
                    "t_dirt"
                }),
                amount: 144,
            }),
            furniture: Vec::new(),
            items: FlatVec(Vec::new()),
            traps: Vec::new(),
            fields: FlatVec(Vec::new()),
            cosmetics: Vec::new(),
            spawns: Vec::new(),
            vehicles: Vec::new(),
            partial_constructions: Vec::new(),
            computers: None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Furniture {
    #[allow(unused)]
    id: ObjectId,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CddaItem {
    pub(crate) typeid: ObjectId,
    snip_id: Option<String>,
    pub(crate) charges: Option<u32>,
    active: Option<bool>,
    pub(crate) corpse: Option<ObjectId>,
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
    pub(crate) id: ObjectId,

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