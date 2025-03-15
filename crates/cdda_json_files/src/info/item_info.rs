use crate::{Flags, HashMap, InfoId, Quality, VecLinkedLater, structure::MaybeFlatVec};
use serde::Deserialize;
use std::sync::Arc;
use units::{Duration, Mass, Volume};

pub trait ItemWithCommonInfo {
    fn common(&self) -> Arc<CommonItemInfo>;
}

#[derive(Debug, Deserialize)]
pub struct Ammo {
    pub ammo_type: Option<MaybeFlatVec<InfoId>>,
    pub casing: Option<InfoId>,
    pub critical_multiplier: Option<u8>,

    // example: { "damage_type": "bullet", "amount": 28, "armor_penetration": 4 }
    pub damage: Option<serde_json::Value>,

    pub drop: Option<Arc<str>>,
    pub effects: Option<Vec<Arc<str>>>,
    pub projectile_count: Option<u32>,

    // example: { "damage_type": "bullet", "amount": 28, "armor_penetration": 4 }
    pub shot_damage: Option<serde_json::Value>,

    // example: { "damage_type": "bullet", "amount": 28, "armor_penetration": 4 }
    pub shot_spread: Option<u16>,

    pub show_stats: Option<bool>,

    #[serde(flatten)]
    pub common: Arc<CommonItemInfo>,
}

impl ItemWithCommonInfo for Ammo {
    fn common(&self) -> Arc<CommonItemInfo> {
        self.common.clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct BionicItem {
    pub difficulty: u8,
    pub installation_data: Option<Arc<str>>,
    pub is_upgrade: bool,

    #[serde(flatten)]
    pub common: Arc<CommonItemInfo>,
}

impl ItemWithCommonInfo for BionicItem {
    fn common(&self) -> Arc<CommonItemInfo> {
        self.common.clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct Book {
    #[serde(default)]
    pub intelligence: u8,

    pub skill: Option<Arc<str>>,
    /// Refers to skill
    #[serde(default)]
    pub required_level: u8,
    /// Refers to skill
    pub max_level: Option<u8>,

    pub chapters: Option<u8>,
    pub time: Duration,

    // map or list of tuples
    pub proficiencies: Option<serde_json::Value>,

    pub martial_art: Option<Arc<str>>,

    #[serde(flatten)]
    pub common: Arc<CommonItemInfo>,
}

impl ItemWithCommonInfo for Book {
    fn common(&self) -> Arc<CommonItemInfo> {
        self.common.clone()
    }
}

/// 'ARMOR' in CDDA
#[derive(Debug, Deserialize)]
pub struct Clothing {
    pub non_functional: Option<Arc<str>>,

    #[serde(flatten)]
    pub common: Arc<CommonItemInfo>,
}

impl ItemWithCommonInfo for Clothing {
    fn common(&self) -> Arc<CommonItemInfo> {
        self.common.clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct Comestible {
    pub addiction_potential: Option<u8>,
    pub addiction_type: Option<Arc<str>>,

    // example: { "ammo_type": "water" }
    pub ammo_data: Option<HashMap<String, String>>,

    pub brewable: Option<HashMap<String, serde_json::Value>>,

    #[serde(default)]
    pub calories: u16,

    pub charges: Option<u16>,
    pub comestible_type: Arc<str>,

    // example: [ { "disease": "bad_food", "probability": 5 } ]
    pub contamination: Option<Arc<[HashMap<String, serde_json::Value>]>>,

    pub cooks_like: Option<Arc<str>>,

    // example: { "type": "emit_actor", "emits": [ "emit_acid_drop" ], "scale_qty": true }
    pub drop_action: Option<HashMap<String, serde_json::Value>>,

    #[serde(default)]
    pub fatigue_mod: i8,

    pub freezing_point: Option<f32>,

    #[serde(default)]
    pub fun: i8,

    #[serde(default)]
    pub healthy: i8,

    pub parasites: Option<u8>,

    pub petfood: Option<serde_json::Value>,
    pub primary_material: Option<Arc<str>>,
    pub rot_spawn: Option<InfoId>,
    pub rot_spawn_chance: Option<u8>,
    pub smoking_result: Option<InfoId>,

    // Duration as String or u16
    pub spoils_in: Option<serde_json::Value>,

    #[serde(default)]
    pub stim: i8,

    pub tool: Option<InfoId>,
    pub vitamins: Option<Arc<[(String, u16)]>>,

    #[serde(flatten)]
    pub common: Arc<CommonItemInfo>,
}

impl ItemWithCommonInfo for Comestible {
    fn common(&self) -> Arc<CommonItemInfo> {
        self.common.clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct Engine {
    pub displacement: Option<u16>,

    #[serde(flatten)]
    pub common: Arc<CommonItemInfo>,
}

impl ItemWithCommonInfo for Engine {
    fn common(&self) -> Arc<CommonItemInfo> {
        self.common.clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct GenericItem {
    pub damage_states: Option<(u8, u8)>,
    pub insulation: Option<u8>,
    pub nanofab_template_group: Option<Arc<str>>,
    pub stackable: Option<bool>,
    pub template_requirements: Option<Arc<str>>,

    #[serde(flatten)]
    pub common: Arc<CommonItemInfo>,
}

impl ItemWithCommonInfo for GenericItem {
    fn common(&self) -> Arc<CommonItemInfo> {
        self.common.clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct Gun {
    pub ammo: Option<MaybeFlatVec<InfoId>>,
    pub ammo_to_fire: Option<u8>,
    pub barrel_volume: Option<Volume>,
    pub blackpowder_tolerance: Option<u8>,

    #[serde(default)]
    pub built_in_mods: Arc<[InfoId]>,

    #[serde(default)]
    pub default_mods: Arc<[InfoId]>,

    pub clip_size: Option<u8>,
    pub durability: Option<u16>,
    pub handling: Option<u8>,

    #[serde(default)]
    pub magazines: Arc<[serde_json::Value]>,

    pub min_cycle_recoil: Option<u16>,

    #[serde(default)]
    pub min_strength: u8,

    #[serde(default)]
    pub modes: Arc<[serde_json::Value]>,

    pub ranged_damage: Option<HashMap<String, serde_json::Value>>,
    pub reload: Option<u16>,
    pub reload_noise: Option<Arc<str>>,
    pub reload_noise_volume: Option<u8>,
    pub ups_charges: Option<u8>,

    #[serde(default)]
    pub valid_mod_locations: Arc<[serde_json::Value]>,

    #[serde(flatten)]
    pub common: Arc<CommonItemInfo>,
}

impl ItemWithCommonInfo for Gun {
    fn common(&self) -> Arc<CommonItemInfo> {
        self.common.clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct Gunmod {
    pub add_mod: Option<Arc<[serde_json::Value]>>,

    #[serde(default)]
    pub aim_speed_modifier: i8,

    pub ammo_modifier: Option<Arc<[String]>>,
    pub consume_chance: Option<u16>,
    pub consume_divisor: Option<u16>,
    pub damage_modifier: Option<serde_json::Value>,

    #[serde(default)]
    pub dispersion_modifier: i16,

    /// In degrees
    pub field_of_view: Option<u16>,

    pub gun_data: Option<HashMap<String, serde_json::Value>>,

    #[serde(default)]
    pub handling_modifier: i8,

    pub install_time: Duration,

    #[serde(default)]
    pub integral_volume: Volume,
    #[serde(default)]
    pub integral_weight: Mass,

    pub location: Arc<str>,

    #[serde(default)]
    pub loudness_modifier: i8,

    pub mod_targets: Arc<[String]>,
    pub mode_modifier: Option<Arc<[serde_json::Value]>>,
    pub overwrite_min_cycle_recoil: Option<u16>,

    #[serde(default)]
    pub range_modifier: i8,

    pub range_multiplier: Option<f32>,
    pub shot_spread_multiplier_modifier: Option<f32>,
    pub ups_charges_multiplier: Option<f32>,
    pub weight_multiplier: Option<f32>,

    #[serde(flatten)]
    pub common: Arc<CommonItemInfo>,
}

impl ItemWithCommonInfo for Gunmod {
    fn common(&self) -> Arc<CommonItemInfo> {
        self.common.clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct Magazine {
    pub ammo_type: Option<MaybeFlatVec<InfoId>>,
    pub capacity: Option<u32>,
    pub default_ammo: Option<InfoId>,
    pub linkage: Option<InfoId>,
    pub reload_time: Option<u16>,

    #[serde(flatten)]
    pub common: Arc<CommonItemInfo>,
}

impl ItemWithCommonInfo for Magazine {
    fn common(&self) -> Arc<CommonItemInfo> {
        self.common.clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct PetArmor {
    pub max_pet_vol: Volume,
    pub min_pet_vol: Volume,
    pub pet_bodytype: Arc<str>,

    #[serde(flatten)]
    pub common: Arc<CommonItemInfo>,
}

impl ItemWithCommonInfo for PetArmor {
    fn common(&self) -> Arc<CommonItemInfo> {
        self.common.clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct Tool {
    pub ammo: Option<MaybeFlatVec<InfoId>>,
    pub charge_factor: Option<u8>,

    #[serde(default)]
    pub charged_qualities: Arc<[serde_json::Value]>,

    #[serde(default)]
    pub initial_charges: u16,

    pub max_charges: Option<u16>,
    pub rand_charges: Option<Arc<[u32]>>,
    pub revert_msg: Option<Arc<str>>,
    pub sub: Option<InfoId>,
    pub variables: Option<HashMap<String, serde_json::Value>>,

    #[serde(flatten)]
    pub common: Arc<CommonItemInfo>,
}

impl ItemWithCommonInfo for Tool {
    fn common(&self) -> Arc<CommonItemInfo> {
        self.common.clone()
    }
}

#[allow(clippy::doc_markdown)]
/// 'TOOL_ARMOR' in CDDA
#[derive(Debug, Deserialize)]
pub struct ToolClothing {
    pub environmental_protection_with_filter: Option<u8>,
    pub weight_capacity_bonus: Option<Mass>,

    #[serde(flatten)]
    pub clothing: Clothing,
}

impl ItemWithCommonInfo for ToolClothing {
    fn common(&self) -> Arc<CommonItemInfo> {
        self.clothing.common.clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct Toolmod {
    pub pocket_mods: Option<Arc<[HashMap<String, serde_json::Value>]>>,

    #[serde(flatten)]
    pub common: Arc<CommonItemInfo>,
}

impl ItemWithCommonInfo for Toolmod {
    fn common(&self) -> Arc<CommonItemInfo> {
        self.common.clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct Wheel {
    pub diameter: u8,
    pub width: u8,

    #[serde(flatten)]
    pub common: Arc<CommonItemInfo>,
}

impl ItemWithCommonInfo for Wheel {
    fn common(&self) -> Arc<CommonItemInfo> {
        self.common.clone()
    }
}

#[derive(Debug, Deserialize)]
pub struct CommonItemInfo {
    pub id: InfoId,
    pub category: Option<Arc<str>>,

    // example: { "price": 0.7, "damage": { "damage_type": "bullet", "amount": 0.9 }, "dispersion": 1.1 }
    pub proportional: Option<serde_json::Value>,

    // example: { "damage": { "damage_type": "bullet", "amount": -1, "armor_penetration": 2 } }
    pub relative: Option<serde_json::Value>,

    pub count: Option<u32>,
    pub stack_size: Option<u8>,
    pub range: Option<i16>, // examples: -6, 140
    pub dispersion: Option<u16>,
    pub recoil: Option<u16>,
    pub loudness: Option<u16>,

    // The fields below are listed in load_basic_info as item_factory.cpp:3932
    #[serde(rename = "weight")]
    pub mass: Option<Mass>,

    #[serde(rename = "integral_weight")]
    pub integral_mass: Option<serde_json::Value>,

    pub volume: Option<Volume>,
    pub longest_side: Option<Arc<str>>,
    pub price: Option<Price>,
    pub price_postapoc: Option<Price>,
    pub integral_volume: Option<serde_json::Value>,
    pub integral_longest_side: Option<serde_json::Value>,
    pub bashing: Option<u16>,
    pub cutting: Option<u16>,
    pub to_hit: Option<ToHit>,
    pub variant_type: Option<serde_json::Value>,
    pub variants: Option<serde_json::Value>,
    pub container: Option<Arc<str>>,
    pub sealed: Option<bool>,
    pub emits: Option<serde_json::Value>,
    pub explode_in_fire: Option<bool>,
    pub solar_efficiency: Option<serde_json::Value>,
    pub ascii_picture: Option<serde_json::Value>,
    pub thrown_damage: Option<serde_json::Value>,
    pub repairs_like: Option<serde_json::Value>,
    pub weapon_category: Option<serde_json::Value>,
    pub degradation_multiplier: Option<serde_json::Value>,

    #[serde(rename(deserialize = "type"))]
    pub type_: Arc<str>,

    pub name: ItemName,
    pub description: Option<Description>,
    pub symbol: Option<char>,
    pub color: Option<Arc<str>>,
    pub material: Option<MaybeFlatVec<Material>>,
    pub material_thickness: Option<f32>,
    pub chat_topics: Option<serde_json::Value>,
    pub phase: Option<Arc<str>>,
    pub magazines: Option<serde_json::Value>,
    pub min_skills: Option<serde_json::Value>,
    pub explosion: Option<serde_json::Value>,
    pub flags: Flags,
    pub faults: Option<serde_json::Value>,

    #[serde(default)]
    pub qualities: VecLinkedLater<Quality, i8>,

    // example: { "effects": [ "RECYCLED" ] }
    pub extend: Option<serde_json::Value>,

    // example: { "effects": [ "NEVER_MISFIRES" ], "flags": [ "IRREPLACEABLE_CONSUMABLE" ] }
    pub delete: Option<serde_json::Value>,

    pub properties: Option<serde_json::Value>,
    pub techniques: Option<serde_json::Value>,
    pub max_charges: Option<u16>,
    pub initial_charges: Option<u16>,
    pub use_action: Option<serde_json::Value>,
    pub countdown_interval: Option<serde_json::Value>,
    pub countdown_destroy: Option<serde_json::Value>,
    pub countdown_action: Option<serde_json::Value>,
    pub looks_like: Option<InfoId>,
    pub conditional_names: Option<serde_json::Value>,
    pub armor_data: Option<serde_json::Value>,
    pub pet_armor_data: Option<serde_json::Value>,
    pub gun_data: Option<serde_json::Value>,
    pub bionic_data: Option<serde_json::Value>,
    pub seed_data: Option<serde_json::Value>,
    pub relic_data: Option<serde_json::Value>,
    pub milling: Option<serde_json::Value>,
    pub gunmod_data: Option<serde_json::Value>,
    pub pocket_data: Option<Vec<serde_json::Value>>,
    pub armor: Option<Vec<serde_json::Value>>,
    pub snippet_category: Option<serde_json::Value>,

    // Plenty of fields already availalble
    #[serde(flatten)]
    pub extra: HashMap<Arc<str>, serde_json::Value>,
}

impl CommonItemInfo {
    #[must_use]
    pub fn melee_damage(&self) -> u16 {
        self.bashing.unwrap_or(0).max(self.cutting.unwrap_or(0))
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum CddaItemName {
    Simple(Arc<str>),
    Both {
        str_sp: Arc<str>,

        ctxt: Option<Arc<str>>,
    },
    Split {
        str: Arc<str>,
        str_pl: Option<Arc<str>>,

        ctxt: Option<Arc<str>>,

        #[serde(rename(deserialize = "//~"))]
        comment: Option<Arc<str>>,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(from = "CddaItemName")]
pub struct ItemName {
    pub single: Arc<str>,
    plural: Arc<str>,
}

impl ItemName {
    #[must_use]
    pub const fn amount(&self, amount: u32) -> &Arc<str> {
        if amount == 1 {
            &self.single
        } else {
            &self.plural
        }
    }
}

impl From<CddaItemName> for ItemName {
    fn from(origin: CddaItemName) -> Self {
        match origin {
            CddaItemName::Simple(string) => Self {
                single: string.clone(),
                plural: (String::from(&*string) + "s").into(),
            },
            CddaItemName::Both { str_sp, .. } => Self {
                single: str_sp.clone(),
                plural: str_sp,
            },
            CddaItemName::Split { str, str_pl, .. } => Self {
                single: str.clone(),
                plural: str_pl.unwrap_or_else(|| (String::from(&*str) + "s").into()),
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Material {
    Simple(Arc<str>),
    Complex {
        #[serde(rename(deserialize = "type"))]
        type_: Arc<str>,

        /// assume 1 when missing
        // TODO What does a fractional value mean?
        portion: Option<f32>,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Price {
    Numeric(u64),
    Text(Arc<str>),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum ToHit {
    Simple(i16),
    Complex(HashMap<Arc<str>, Arc<str>>),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum Description {
    Simple(Arc<str>),
    Complex(HashMap<Arc<str>, Arc<str>>),
}

#[cfg(test)]
mod item_tests {
    use super::*;
    #[test]
    fn ghee_works() {
        let json = include_str!("test_ghee.json");
        let result = serde_json::from_str::<CommonItemInfo>(json);
        assert!(result.is_ok(), "{result:?}");
    }
    #[test]
    fn mc_jian_works() {
        let json = include_str!("test_mc_jian.json");
        let result = serde_json::from_str::<CommonItemInfo>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
