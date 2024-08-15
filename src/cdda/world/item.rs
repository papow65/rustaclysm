use crate::gameplay::ObjectId;
use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CddaItem {
    pub(crate) typeid: ObjectId,

    #[allow(unused)]
    pub(crate) snip_id: Option<String>,

    pub(crate) charges: Option<u32>,

    #[allow(unused)]
    pub(crate) active: Option<bool>,

    pub(crate) corpse: Option<ObjectId>,

    #[allow(unused)]
    pub(crate) name: Option<String>,
    #[allow(unused)]
    pub(crate) owner: Option<String>,
    #[allow(unused)]
    pub(crate) bday: Option<i64>,
    #[allow(unused)]
    pub(crate) last_temp_check: Option<u64>,
    #[allow(unused)]
    pub(crate) specific_energy: Option<Number>,
    #[allow(unused)]
    pub(crate) temperature: Option<Number>,
    #[allow(unused)]
    pub(crate) item_vars: Option<HashMap<String, String>>,
    #[allow(unused)]
    #[serde(default)]
    pub(crate) item_tags: Vec<String>,
    #[allow(unused)]
    pub(crate) contents: Option<CddaContainer>,
    #[allow(unused)]
    #[serde(default)]
    pub(crate) components: Vec<CddaItem>,
    #[allow(unused)]
    pub(crate) is_favorite: Option<bool>,
    #[allow(unused)]
    pub(crate) relic_data: Option<serde_json::Value>,
    #[allow(unused)]
    pub(crate) damaged: Option<i64>,
    #[allow(unused)]
    pub(crate) current_phase: Option<u8>,
    #[allow(unused)]
    #[serde(default)]
    pub(crate) faults: Vec<String>,
    #[allow(unused)]
    pub(crate) rot: Option<i64>,
    #[allow(unused)]
    pub(crate) curammo: Option<String>,
    #[allow(unused)]
    pub(crate) item_counter: Option<u8>,
    #[allow(unused)]
    pub(crate) variant: Option<String>,
    #[allow(unused)]
    pub(crate) recipe_charges: Option<u8>,
    #[allow(unused)]
    pub(crate) poison: Option<u8>,
    #[allow(unused)]
    pub(crate) burnt: Option<serde_json::Value>,
    #[allow(unused)]
    pub(crate) craft_data: Option<serde_json::Value>,
    #[allow(unused)]
    pub(crate) dropped_from: Option<String>,
    #[allow(unused)]
    pub(crate) degradation: Option<u32>,
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
pub(crate) struct CddaContainer {
    #[allow(unused)]
    contents: Vec<Pocket>,

    #[allow(unused)]
    additional_pockets: Option<Vec<AdditionalPocket>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Pocket {
    #[allow(unused)]
    pocket_type: u8,

    #[allow(unused)]
    contents: Vec<CddaItem>,

    #[allow(unused)]
    _sealed: bool,

    #[allow(unused)]
    allowed: Option<bool>,

    #[allow(unused)]
    favorite_settings: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AdditionalPocket {
    #[allow(unused)]
    pub(crate) typeid: ObjectId,

    #[allow(unused)]
    last_temp_check: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum Number {
    Int(#[allow(dead_code)] i64),
    Text(#[allow(dead_code)] String),
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
