use crate::{
    CharacterInfo, CommonItemInfo, OptionalLinkedLater, PocketType, RequiredLinkedLater,
    UntypedInfoId,
};
use bevy_platform::collections::HashMap;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use serde_json::Value as JsonValue;
use std::sync::{Arc, Mutex};
use strum::VariantArray as _;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CddaItem {
    #[serde(rename = "typeid")]
    pub item_info: RequiredLinkedLater<CommonItemInfo>,

    /// Can change after a migration
    pub variant: Mutex<Option<Arc<str>>>,

    pub snip_id: Option<Arc<str>>,
    pub charges: Option<u32>,
    pub active: Option<bool>,

    pub corpse: OptionalLinkedLater<CharacterInfo>,

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

    // Sometilmes Vec<CddaItem>, sometimes HashMap<String, CddaItem>
    pub components: Option<JsonValue>,

    pub is_favorite: Option<bool>,
    pub relic_data: Option<JsonValue>,
    pub damaged: Option<i64>,
    pub current_phase: Option<u8>,

    #[serde(default)]
    pub faults: Vec<Arc<str>>,

    pub rot: Option<i64>,
    pub curammo: Option<Arc<str>>,
    pub item_counter: Option<u8>,
    pub recipe_charges: Option<u8>,
    pub poison: Option<u8>,
    pub burnt: Option<JsonValue>,
    pub craft_data: Option<JsonValue>,
    pub dropped_from: Option<Arc<str>>,
    pub degradation: Option<u32>,
    pub link_data: Option<HashMap<Arc<str>, JsonValue>>,
    pub invlet: Option<u8>,
}

impl CddaItem {
    #[must_use]
    pub fn new(item_info: &Arc<CommonItemInfo>) -> Self {
        Self {
            item_info: RequiredLinkedLater::new_final(item_info.id.clone(), item_info),
            snip_id: None,
            charges: None,
            active: None,
            corpse: OptionalLinkedLater::new_final_none(),
            name: None,
            owner: None,
            bday: None,
            last_temp_check: None,
            specific_energy: None,
            temperature: None,
            item_vars: None,
            item_tags: Vec::new(),
            contents: None,
            components: None,
            is_favorite: None,
            relic_data: None,
            damaged: None,
            current_phase: None,
            faults: Vec::new(),
            rot: None,
            curammo: None,
            item_counter: None,
            variant: None.into(),
            recipe_charges: None,
            poison: None,
            burnt: None,
            craft_data: None,
            dropped_from: None,
            degradation: None,
            link_data: None,
            invlet: None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CddaContainer {
    pub contents: Vec<CddaPocket>,

    #[expect(unused)]
    #[serde(default)]
    additional_pockets: Vec<AdditionalPocket>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CddaPocket {
    #[serde(deserialize_with = "pocket_type_from_index")]
    pub pocket_type: PocketType,

    pub contents: Vec<CddaItem>,

    #[serde(rename = "_sealed")]
    pub sealed: bool,

    /// TODO Related to stealing? Also set on (some?) corpses
    #[serde(default = "always_true")]
    pub allowed: bool,

    #[expect(unused)]
    favorite_settings: Option<JsonValue>,

    #[expect(unused)]
    no_rigid: Option<JsonValue>,
}

fn pocket_type_from_index<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<PocketType, D::Error> {
    Ok(*PocketType::VARIANTS
        .get(usize::deserialize(deserializer)?)
        .ok_or_else(|| Error::custom("Pocket type index out of range"))?)
}

const fn always_true() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdditionalPocket {
    pub typeid: UntypedInfoId,

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
    use serde_json::from_str as from_json_str;

    #[test]
    fn it_works() {
        let json = include_str!("test_container.json");
        let result = from_json_str::<CddaContainer>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
