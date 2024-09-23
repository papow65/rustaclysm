use crate::{Flags, ItemName, ObjectId};
use bevy::utils::HashMap;
use serde::Deserialize;
use units::{Mass, Volume};

#[derive(Debug, Deserialize)]
pub struct CharacterInfo {
    pub name: ItemName,
    pub default_faction: String,
    pub looks_like: Option<ObjectId>,
    pub volume: Option<Volume>,

    #[serde(rename = "weight")]
    pub mass: Option<Mass>,

    pub hp: Option<u32>,

    #[serde(default)]
    pub speed: u64,

    #[serde(default)]
    pub melee_dice: u16,

    #[serde(default)]
    pub melee_dice_sides: u16,

    #[serde(default)]
    pub flags: Flags,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod character_tests {
    use super::*;
    #[test]
    fn it_works() {
        let json = include_str!("test_mon_bee.json");
        let result = serde_json::from_str::<CharacterInfo>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
