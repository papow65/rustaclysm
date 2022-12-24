use crate::prelude::{Flags, ItemName};
use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub(crate) enum CddaTerrainInfo {
    #[serde(rename(deserialize = "terrain"))]
    Terrain {
        name: ItemName,
        move_cost: MoveCost,
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

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, PartialOrd)]
pub(crate) struct MoveCost(pub(crate) u8);

impl MoveCost {
    pub(crate) fn adjust(&self, cost_mod: Option<MoveCostMod>) -> Self {
        let extra = cost_mod.map_or(0, |c| c.0);
        assert!(0 <= extra);
        Self(self.0 + extra as u8)
    }
}

impl Default for MoveCost {
    fn default() -> Self {
        Self(2)
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, PartialOrd)]
pub(crate) struct MoveCostMod(pub(crate) i8);

#[derive(Debug, Deserialize)]
pub(crate) struct IntensityLevel {
    name: Option<ItemName>,

    #[allow(unused)]
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}
