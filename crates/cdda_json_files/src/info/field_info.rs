use crate::{Ignored, InfoId, ItemName, UntypedInfoId};
use bevy_platform::collections::HashMap;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub struct FieldInfo {
    pub id: InfoId<Self>,
    pub intensity_levels: Vec<IntensityLevel>,
    pub looks_like: Option<UntypedInfoId>,

    pub accelerated_decay: Option<bool>,
    pub apply_slime_factor: Option<u8>,
    pub decay_amount_factor: Option<u8>,
    pub description_affix: Option<Arc<str>>,
    pub dirty_transparency_cache: Option<bool>,

    #[serde(default = "get_true")]
    pub display_field: bool,

    #[serde(default = "get_true")]
    pub display_items: bool,

    pub gas_absorption_factor: Option<u8>,
    pub half_life: Option<Arc<str>>,
    pub has_acid: Option<bool>,
    pub has_elec: Option<bool>,
    pub has_fire: Option<bool>,
    pub has_fume: Option<bool>,
    pub immunity_data: Option<serde_json::Value>,

    #[serde(default)]
    pub is_splattering: bool,

    pub legacy_enum_id: Option<u8>,
    pub legacy_make_rubble: Option<bool>,
    pub mopsafe: Option<bool>,
    pub npc_complain: Option<serde_json::Value>,
    pub outdoor_age_speedup: Option<Arc<str>>,
    pub percent_spread: Option<u8>,
    pub phase: Option<Arc<str>>,
    pub priority: Option<i8>,
    pub underwater_age_speedup: Option<Arc<str>>,
    pub wandering_field: Option<Arc<str>>,

    pub bash: Option<serde_json::Value>,
    pub decrease_intensity_on_contact: Option<bool>,
    pub immune_mtypes: Option<Vec<serde_json::Value>>,

    #[serde(flatten)]
    _ignored: Ignored<Self>,
}

impl FieldInfo {
    #[must_use]
    pub fn name(&self) -> &ItemName {
        self.intensity_levels[0]
            .name
            .as_ref()
            .expect("Named first level")
    }
}

#[derive(Debug, Deserialize)]
pub struct IntensityLevel {
    name: Option<ItemName>,

    #[expect(unused)]
    #[serde(flatten)]
    extra: HashMap<Arc<str>, serde_json::Value>,
}

const fn get_true() -> bool {
    true
}
