use crate::prelude::{Flags, ItemName, ObjectId};
use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct CharacterInfo {
    pub(crate) name: ItemName,
    pub(crate) looks_like: Option<ObjectId>,

    #[serde(default)]
    pub(crate) flags: Flags,

    #[allow(unused)]
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}
