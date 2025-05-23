use crate::{Flags, Ignored, InfoId, ItemName, UntypedInfoId};
use serde::Deserialize;
use std::sync::Arc;
use units::{Mass, Volume};

#[derive(Debug, Deserialize)]
pub struct CharacterInfo {
    pub id: InfoId<Self>,
    pub name: ItemName,
    pub symbol: char,
    pub hp: u32,
    pub default_faction: Arc<str>,
    pub description: Arc<str>,

    pub looks_like: Option<UntypedInfoId>,
    pub volume: Option<Volume>,

    #[serde(rename = "weight")]
    pub mass: Option<Mass>,

    #[serde(default)]
    pub speed: u64,

    #[serde(default)]
    pub melee_dice: u16,

    #[serde(default)]
    pub melee_dice_sides: u16,

    #[serde(default)]
    pub flags: Flags,

    pub absorb_ml_per_hp: Option<u8>,
    pub absorb_move_cost_max: Option<u16>,
    pub absorb_move_cost_per_ml: Option<f32>,
    pub aggression: Option<i8>,
    pub aggro_character: Option<bool>,
    pub anger_triggers: Option<Vec<serde_json::Value>>,
    pub armor_acid: Option<u16>,
    pub armor_bash: Option<u16>,
    pub armor_bullet: Option<u16>,
    pub armor_cold: Option<u16>,
    pub armor_cut: Option<u16>,
    pub armor_elec: Option<u16>,
    pub armor_fire: Option<u16>,
    pub armor_stab: Option<u16>,

    #[serde(alias = "attack_cost ")]
    pub attack_cost: Option<u16>,

    pub attack_effs: Option<Vec<serde_json::Value>>,
    pub baby_flags: Flags,
    pub biosignature: Option<serde_json::Value>,
    pub bleed_rate: Option<u8>,
    pub bodytype: Option<Arc<str>>,
    pub burn_into: Option<Arc<str>>,
    pub categories: Option<Vec<serde_json::Value>>,
    pub color: Option<Arc<str>>,
    pub colour: Option<Arc<str>>,
    pub death_drops: Option<serde_json::Value>,
    pub death_function: Option<serde_json::Value>,
    pub delete: Option<serde_json::Value>,
    pub diff: Option<u8>,
    pub dissect: Option<Arc<str>>,
    pub dodge: Option<u8>,
    pub emit_fields: Option<Vec<serde_json::Value>>,
    pub extend: Option<serde_json::Value>,
    pub families: Option<Vec<serde_json::Value>>,
    pub fear_triggers: Option<Vec<serde_json::Value>>,
    pub fungalize_into: Option<Arc<str>>,
    pub grab_strength: Option<u8>,
    pub harvest: Option<Arc<str>>,
    pub luminance: Option<u16>,
    pub material: Option<Vec<serde_json::Value>>,
    pub mech_battery: Option<Arc<str>>,
    pub mech_str_bonus: Option<u8>,
    pub mech_weapon: Option<Arc<str>>,
    pub melee_damage: Option<Vec<serde_json::Value>>,
    pub melee_skill: Option<u8>,
    pub melee_training_cap: Option<u8>,
    pub morale: Option<i16>,
    pub mountable_weight_ratio: Option<u8>,
    pub path_settings: Option<serde_json::Value>,
    pub petfood: Option<serde_json::Value>,
    pub phase: Option<Arc<str>>,
    pub placate_triggers: Option<Vec<serde_json::Value>>,
    pub proportional: Option<serde_json::Value>,
    pub regen_morale: Option<bool>,
    pub regenerates: Option<u8>,
    pub regenerates_in_dark: Option<bool>,
    pub regeneration_modifiers: Option<Vec<serde_json::Value>>,
    pub relative: Option<serde_json::Value>,
    pub reproduction: Option<serde_json::Value>,
    pub revert_to_itype: Option<Arc<str>>,
    pub scents_ignored: Option<Vec<serde_json::Value>>,
    pub shearing: Option<Vec<serde_json::Value>>,
    pub special_attacks: Option<Vec<serde_json::Value>>,
    pub special_when_hit: Option<Vec<serde_json::Value>>,
    pub species: Option<Vec<serde_json::Value>>,
    pub split_move_cost: Option<u8>,
    pub starting_ammo: Option<serde_json::Value>,
    pub tracking_distance: Option<u8>,
    pub upgrades: Option<serde_json::Value>,
    pub vision_day: Option<u8>,
    pub vision_night: Option<u8>,
    pub weakpoint_sets: Option<Vec<serde_json::Value>>,
    pub weakpoints: Option<Vec<serde_json::Value>>,
    pub zombify_into: Option<Arc<str>>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
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
