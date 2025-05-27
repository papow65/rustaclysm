use crate::{
    CharacterInfo, CommonItemInfo, InfoId, Magazine, OptionalLinkedLater, RequiredLinkedLater,
    UntypedInfoId,
};
use bevy_platform::collections::HashMap;
use serde::Deserialize;
use serde_repr::Deserialize_repr;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CddaItem {
    #[serde(rename = "typeid")]
    pub item_info: RequiredLinkedLater<CommonItemInfo>,

    #[serde(skip)]
    pub magazine_info: OnceLock<OptionalLinkedLater<Magazine>>,

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
    pub recipe_charges: Option<u8>,
    pub poison: Option<u8>,
    pub burnt: Option<serde_json::Value>,
    pub craft_data: Option<serde_json::Value>,
    pub dropped_from: Option<Arc<str>>,
    pub degradation: Option<u32>,
}

impl CddaItem {
    #[must_use]
    pub fn new(item_info: &Arc<CommonItemInfo>, magazine: Option<&Arc<Magazine>>) -> Self {
        let magazine_link = if let Some(magazine) = magazine {
            let magazine_id: InfoId<Magazine> = magazine.common.id.clone().into();
            OptionalLinkedLater::new_final_some(magazine_id, magazine)
        } else {
            OptionalLinkedLater::new_final_none()
        };

        Self {
            item_info: RequiredLinkedLater::new_final(item_info.id.clone(), item_info),
            magazine_info: magazine_link.into(),
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
            components: Vec::new(),
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
    pub pocket_type: PocketType,
    pub contents: Vec<CddaItem>,

    #[serde(rename = "_sealed")]
    pub sealed: bool,

    /// TODO Related to stealing? Also set on (some?) corpses
    #[serde(default = "return_true")]
    pub allowed: bool,

    #[expect(unused)]
    favorite_settings: Option<serde_json::Value>,

    #[expect(unused)]
    no_rigid: Option<serde_json::Value>,
}

const fn return_true() -> bool {
    true
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize_repr)]
#[serde(from = "u8")]
#[repr(u8)]
pub enum PocketType {
    // Order-dependant!
    // Based on item_pocket.h:40-48
    Container,
    Magazine,
    /// Holds magazines
    MagazineWell,
    /// Gunmods or toolmods
    Mod,
    /// Bionics embedded in a corpse
    Corpse,
    Software,
    Ebook,
    /// Allows items to load contents that are too big, in order to spill them later.
    Migration,
    Last,
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
    #[test]
    fn it_works() {
        let json = include_str!("test_container.json");
        let result = serde_json::from_str::<CddaContainer>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
