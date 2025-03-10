use crate::{
    At, AtVec, CddaItem, CddaVehicle, FieldVec, FurnitureInfo, ObjectId, Repetition,
    RepetitionBlock, RequiredLinkedLater, Spawn, TerrainInfo,
};
use serde::Deserialize;
use std::sync::{Arc, OnceLock};

// Reference: https://github.com/CleverRaven/Cataclysm-DDA/blob/master/src/savegame_json.cpp

/// Corresponds to a 'map' in CDDA. It defines the layout of a `ZoneLevel`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Map(pub [Submap; 4]);

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Submap {
    pub version: u64,
    pub coordinates: (i32, i32, i8),
    pub turn_last_touched: u64,
    pub temperature: i64,
    pub radiation: Vec<i64>,
    pub terrain: RepetitionBlock<RequiredLinkedLater<TerrainInfo>>,
    pub furniture: Vec<At<RequiredLinkedLater<FurnitureInfo>>>,
    pub items: AtVec<Vec<Repetition<CddaItem>>>,
    pub traps: Vec<At<ObjectId>>,
    pub fields: AtVec<FieldVec>,
    pub cosmetics: Vec<(u8, u8, Arc<str>, Arc<str>)>,
    pub spawns: Vec<Spawn>,
    pub vehicles: Vec<CddaVehicle>,
    pub partial_constructions: Vec<serde_json::Value>,

    #[serde(default)]
    pub computers: Vec<serde_json::Value>,

    /// Marker to remember the state of the links
    #[serde(skip)]
    pub linked: OnceLock<()>,
}
