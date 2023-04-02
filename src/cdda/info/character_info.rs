use crate::prelude::{Flags, ItemName, Mass, ObjectId, Volume};
use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct CharacterInfo {
    pub(crate) name: ItemName,
    pub(crate) default_faction: String,
    pub(crate) looks_like: Option<ObjectId>,
    pub(crate) volume: Option<Volume>,

    #[serde(rename = "weight")]
    pub(crate) mass: Option<Mass>,

    pub(crate) hp: Option<u32>,

    #[serde(default)]
    pub(crate) speed: u64,

    #[serde(default)]
    pub(crate) melee_dice: u16,

    #[serde(default)]
    pub(crate) melee_dice_sides: u16,

    #[serde(default)]
    pub(crate) flags: Flags,

    #[allow(unused)]
    #[serde(flatten)]
    pub(crate) extra: HashMap<String, serde_json::Value>,
}
