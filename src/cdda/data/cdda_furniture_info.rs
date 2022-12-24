use crate::prelude::{ItemName, MoveCostMod};
use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct CddaFurnitureInfo {
    pub(crate) name: ItemName,
    pub(crate) move_cost_mod: MoveCostMod,

    #[allow(unused)]
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}
