use crate::prelude::{Flags, ItemName, Mass, ObjectId, Volume};
use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct CharacterInfo {
    pub(crate) name: ItemName,
    pub(crate) looks_like: Option<ObjectId>,
    pub(crate) volume: Option<Volume>,

    #[serde(rename = "weight")]
    pub(crate) mass: Option<Mass>,

    #[serde(default)]
    pub(crate) flags: Flags,

    #[allow(unused)]
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}
