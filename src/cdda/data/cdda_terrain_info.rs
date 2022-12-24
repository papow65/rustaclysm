use crate::prelude::{Flags, ItemName};
use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub(crate) enum CddaTerrainInfo {
    #[serde(rename(deserialize = "terrain"))]
    Terrain {
        name: ItemName,

        move_cost: u8,

        coverage: Option<u8>,

        flags: Flags,

        #[allow(unused)]
        #[serde(flatten)]
        extra: HashMap<String, serde_json::Value>,
    },
    #[serde(rename(deserialize = "field_type"))]
    FieldType {
        intensity_levels: Vec<IntensityLevel>,

        #[allow(unused)]
        #[serde(flatten)]
        extra: HashMap<String, serde_json::Value>,
    },
}

impl CddaTerrainInfo {
    pub(crate) fn name(&self) -> &ItemName {
        match self {
            Self::Terrain { name, .. } => name,
            Self::FieldType {
                intensity_levels, ..
            } => intensity_levels[0].name.as_ref().unwrap(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct IntensityLevel {
    name: Option<ItemName>,

    #[allow(unused)]
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}
