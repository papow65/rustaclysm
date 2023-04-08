use crate::prelude::{Flags, ItemName, MoveCostMod, ObjectId};
use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct FurnitureInfo {
    pub(crate) name: ItemName,
    pub(crate) move_cost_mod: MoveCostMod,
    pub(crate) looks_like: Option<ObjectId>,

    #[serde(default)]
    pub(crate) flags: Flags,

    pub(crate) bash: Option<HashMap<String, serde_json::Value>>,

    #[allow(unused)]
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}
