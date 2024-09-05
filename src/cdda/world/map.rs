use crate::cdda::{
    At, AtVec, CddaAmount, CddaItem, CddaVehicle, Error, FieldVec, FlatVec, Repetition,
    RepetitionBlock, Spawn,
};
use crate::common::{PathFor, WorldPath};
use crate::gameplay::{ObjectId, SubzoneLevel, ZoneLevel};
use bevy::asset::{io::Reader, Asset, AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use futures_lite::AsyncReadExt;
use serde::Deserialize;
use std::str::from_utf8;

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

/// Corresponds to a 'map' in CDDA. It defines the layout of a `ZoneLevel`.
#[derive(Debug, Deserialize, Asset, TypePath)]
#[serde(deny_unknown_fields)]
#[type_path = "cdda::world::Map"]
pub(crate) struct Map(pub(crate) [Submap; 4]);

#[derive(Default)]
pub(crate) struct MapLoader;

impl AssetLoader for MapLoader {
    type Asset = Map;
    type Settings = ();
    type Error = Error;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|err| Error::Io { _wrapped: err })?;

        let map = serde_json::from_slice::<Map>(&bytes).map_err(|err| Error::Json {
            _wrapped: err,
            _file_path: load_context.path().to_path_buf(),
            _contents: String::from(from_utf8(&bytes[0..1000]).unwrap_or("(invalid UTF8)")),
        })?;
        Ok(map)
    }

    fn extensions(&self) -> &[&str] {
        &["map"]
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Submap {
    #[expect(unused)]
    version: u64,

    #[expect(unused)]
    pub(crate) coordinates: (i32, i32, i8),

    #[expect(unused)]
    turn_last_touched: u64,

    #[expect(unused)]
    temperature: i64,

    #[expect(unused)]
    radiation: Vec<i64>,

    pub(crate) terrain: RepetitionBlock<ObjectId>,
    pub(crate) furniture: Vec<At<ObjectId>>,
    pub(crate) items: AtVec<Vec<Repetition<CddaItem>>>,

    #[expect(unused)]
    traps: Vec<At<ObjectId>>,

    pub(crate) fields: AtVec<FieldVec>,

    #[expect(unused)]
    cosmetics: Vec<(u8, u8, String, String)>,

    pub(crate) spawns: Vec<Spawn>,
    pub(crate) vehicles: Vec<CddaVehicle>,

    #[expect(unused)]
    partial_constructions: Vec<serde_json::Value>,

    #[expect(unused)]
    #[serde(default)]
    computers: Vec<serde_json::Value>,
}

impl Submap {
    pub(crate) fn fallback(subzone_level: SubzoneLevel, zone_object_id: &ObjectId) -> Self {
        Self {
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
            computers: Vec::new(),
        }
    }
}
