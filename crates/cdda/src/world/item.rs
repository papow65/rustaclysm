use crate::ObjectId;
use bevy::utils::HashMap;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CddaItem {
    pub typeid: ObjectId,

    pub snip_id: Option<Arc<str>>,
    pub charges: Option<u32>,
    pub active: Option<bool>,

    pub corpse: Option<ObjectId>,

    pub name: Option<Arc<str>>,
    pub owner: Option<Arc<str>>,
    pub bday: Option<i64>,
    pub last_temp_check: Option<u64>,
    pub specific_energy: Option<Number>,
    pub temperature: Option<Number>,
    pub item_vars: Option<HashMap<Arc<str>, Arc<str>>>,

    #[serde(default)]
    pub item_tags: Vec<Arc<str>>,

    pub contents: Option<CddaContainer>,

    #[serde(default)]
    pub components: Vec<CddaItem>,

    pub is_favorite: Option<bool>,
    pub relic_data: Option<serde_json::Value>,
    pub damaged: Option<i64>,
    pub current_phase: Option<u8>,

    #[serde(default)]
    pub faults: Vec<Arc<str>>,

    pub rot: Option<i64>,
    pub curammo: Option<Arc<str>>,
    pub item_counter: Option<u8>,
    pub variant: Option<Arc<str>>,
    pub recipe_charges: Option<u8>,
    pub poison: Option<u8>,
    pub burnt: Option<serde_json::Value>,
    pub craft_data: Option<serde_json::Value>,
    pub dropped_from: Option<Arc<str>>,
    pub degradation: Option<u32>,
}

impl From<ObjectId> for CddaItem {
    fn from(typeid: ObjectId) -> Self {
        Self {
            typeid,
            snip_id: None,
            charges: None,
            active: None,
            corpse: None,
            name: None,
            owner: None,
            bday: None,
            last_temp_check: None,
            specific_energy: None,
            temperature: None,
            item_vars: None,
            item_tags: Vec::new(),
            contents: None,
            components: Vec::new(),
            is_favorite: None,
            relic_data: None,
            damaged: None,
            current_phase: None,
            faults: Vec::new(),
            rot: None,
            curammo: None,
            item_counter: None,
            variant: None,
            recipe_charges: None,
            poison: None,
            burnt: None,
            craft_data: None,
            dropped_from: None,
            degradation: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CddaContainer {
    #[expect(unused)]
    contents: Vec<Pocket>,

    #[expect(unused)]
    additional_pockets: Option<Vec<AdditionalPocket>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pocket {
    #[expect(unused)]
    pocket_type: u8,

    #[expect(unused)]
    contents: Vec<CddaItem>,

    _sealed: bool,

    #[expect(unused)]
    allowed: Option<bool>,

    #[expect(unused)]
    favorite_settings: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdditionalPocket {
    #[expect(unused)]
    pub typeid: ObjectId,

    #[expect(unused)]
    last_temp_check: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Number {
    Int(i64),
    Text(Arc<str>),
}

#[cfg(test)]
mod container_tests {
    use super::*;
    #[test]
    fn it_works() {
        let json = include_str!("test_container.json");
        let result = serde_json::from_str::<CddaContainer>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
