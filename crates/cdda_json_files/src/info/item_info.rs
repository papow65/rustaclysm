use crate::{Flags, Ignored, InfoId, ItemQuality, MaybeFlatVec, UntypedInfoId, UseAction};
use bevy_platform::collections::HashMap;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, OnceLock};
use strum::VariantArray;
use units::{Distance, Duration, Mass, Volume};

pub trait ItemWithCommonInfo {
    fn common(&self) -> Arc<CommonItemInfo>;
}

#[derive(Debug, Deserialize)]
pub struct Ammo {
    pub ammo_type: MaybeFlatVec<UntypedInfoId>,

    pub casing: Option<UntypedInfoId>,
    pub critical_multiplier: Option<u8>,

    // example: { "damage_type": "bullet", "amount": 28, "armor_penetration": 4 }
    pub damage: Option<JsonValue>,

    pub drop: Option<Arc<str>>,
    pub effects: Option<Vec<Arc<str>>>,
    pub projectile_count: Option<u32>,

    // example: { "damage_type": "bullet", "amount": 28, "armor_penetration": 4 }
    pub shot_damage: Option<JsonValue>,

    // example: { "damage_type": "bullet", "amount": 28, "armor_penetration": 4 }
    pub shot_spread: Option<u16>,

    pub show_stats: Option<bool>,

    pub charges: Option<u8>,
    pub comestible_type: Option<Arc<str>>,
    pub primary_material: Option<Arc<str>>,

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
    pub is_upgrade: bool,

    pub installation_data: Option<Arc<str>>,

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
    pub time: Duration,

    #[serde(default)]
    pub intelligence: u8,

    #[serde(default)]
    pub fun: i8,

    pub skill: Option<Arc<str>>,
    /// Refers to skill
    #[serde(default)]
    pub required_level: u8,
    /// Refers to skill
    pub max_level: Option<u8>,

    pub chapters: Option<u8>,

    // map or list of tuples
    pub proficiencies: Option<JsonValue>,

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
    pub environmental_protection: Option<u8>,
    pub warmth: Option<u8>,
    pub sided: Option<bool>,
    pub power_armor: Option<bool>,

    #[serde(default)]
    pub covers: Vec<JsonValue>,

    #[serde(default)]
    pub valid_mods: Vec<JsonValue>,

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
    pub comestible_type: Arc<str>,

    #[serde(default)]
    pub calories: u16,

    #[serde(default)]
    pub fun: i8,

    #[serde(default)]
    pub healthy: i8,

    pub addiction_potential: Option<u8>,
    pub addiction_type: Option<Arc<str>>,

    // example: { "ammo_type": "water" }
    pub ammo_data: Option<HashMap<String, String>>,

    pub brewable: Option<HashMap<String, JsonValue>>,

    pub charges: Option<u16>,

    // example: [ { "disease": "bad_food", "probability": 5 } ]
    pub contamination: Option<Arc<[HashMap<String, JsonValue>]>>,

    pub cooks_like: Option<Arc<str>>,

    // example: { "type": "emit_actor", "emits": [ "emit_acid_drop" ], "scale_qty": true }
    pub drop_action: Option<HashMap<String, JsonValue>>,

    #[serde(default)]
    pub fatigue_mod: i8,

    pub freezing_point: Option<f32>,

    pub quench: Option<i8>,
    pub parasites: Option<u8>,

    pub petfood: Option<JsonValue>,
    pub primary_material: Option<Arc<str>>,
    pub rot_spawn: Option<UntypedInfoId>,
    pub rot_spawn_chance: Option<u8>,
    pub smoking_result: Option<UntypedInfoId>,

    // Duration as String or u16
    pub spoils_in: Option<JsonValue>,

    #[serde(default)]
    pub stim: i8,

    pub tool: Option<UntypedInfoId>,
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

    pub calories: Option<u16>,
    pub comestible_type: Option<Arc<str>>,
    pub vitamins: Option<Arc<[(String, u16)]>>,
    pub fun: Option<i8>,

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
    pub skill: Arc<str>,

    pub ammo: Option<MaybeFlatVec<UntypedInfoId>>,
    pub ammo_to_fire: Option<u8>,
    pub barrel_volume: Option<Volume>,
    pub blackpowder_tolerance: Option<u8>,

    #[serde(default)]
    pub built_in_mods: Arc<[UntypedInfoId]>,

    #[serde(default)]
    pub default_mods: Arc<[UntypedInfoId]>,

    pub clip_size: Option<u8>,
    pub durability: Option<u16>,
    pub handling: Option<u8>,

    #[serde(default)]
    pub magazines: Arc<[JsonValue]>,

    pub min_cycle_recoil: Option<u16>,

    #[serde(default)]
    pub min_strength: u8,

    #[serde(default)]
    pub modes: Arc<[JsonValue]>,

    pub ranged_damage: Option<JsonValue>,
    pub reload: Option<u16>,
    pub reload_noise: Option<Arc<str>>,
    pub reload_noise_volume: Option<u8>,
    pub ups_charges: Option<u8>,

    #[serde(default)]
    pub ammo_effects: Vec<Arc<str>>,

    pub sight_dispersion: Option<u16>,

    #[serde(default)]
    pub valid_mod_locations: Arc<[JsonValue]>,

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
    pub location: Arc<str>,
    pub mod_targets: Arc<[String]>,
    pub install_time: Duration,

    pub add_mod: Option<Arc<[JsonValue]>>,

    #[serde(default)]
    pub aim_speed_modifier: i8,

    #[serde(default)]
    pub dispersion_modifier: i16,

    #[serde(default)]
    pub handling_modifier: i8,

    #[serde(default)]
    pub loudness_modifier: i8,

    #[serde(default)]
    pub range_modifier: i8,

    pub ammo_modifier: Option<Arc<[String]>>,
    pub consume_chance: Option<u16>,
    pub consume_divisor: Option<u16>,
    pub damage_modifier: Option<JsonValue>,

    pub sight_dispersion: Option<u16>,

    /// In degrees
    pub field_of_view: Option<u16>,

    pub gun_data: Option<HashMap<String, JsonValue>>,

    #[serde(default)]
    pub integral_volume: Volume,
    #[serde(default)]
    pub integral_weight: Mass,

    pub mode_modifier: Option<Arc<[JsonValue]>>,
    pub overwrite_min_cycle_recoil: Option<u16>,

    pub range_multiplier: Option<f32>,
    pub shot_spread_multiplier_modifier: Option<f32>,
    pub ups_charges_multiplier: Option<f32>,
    pub weight_multiplier: Option<f32>,

    #[serde(default)]
    pub ammo_effects: Vec<JsonValue>,

    #[serde(default)]
    pub acceptable_ammo: Vec<JsonValue>,

    #[serde(default)]
    pub magazine_adaptor: Vec<JsonValue>,

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
    pub ammo_type: MaybeFlatVec<UntypedInfoId>,
    pub capacity: Option<u32>,
    pub default_ammo: Option<UntypedInfoId>,
    pub linkage: Option<UntypedInfoId>,
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
    pub pet_bodytype: Arc<str>,
    pub max_pet_vol: Volume,
    pub min_pet_vol: Volume,

    pub environmental_protection: Option<u8>,

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
    pub ammo: Option<MaybeFlatVec<UntypedInfoId>>,
    pub charge_factor: Option<u8>,

    #[serde(default)]
    pub charged_qualities: Arc<[JsonValue]>,

    #[serde(default)]
    pub initial_charges: u16,

    pub charges_per_use: Option<u16>,
    pub turns_per_charge: Option<u16>,
    pub max_charges: Option<u16>,
    pub rand_charges: Option<Arc<[u32]>>,
    pub power_draw: Option<Arc<str>>,
    pub revert_to: Option<Arc<str>>,
    pub revert_msg: Option<Arc<str>>,
    pub sub: Option<UntypedInfoId>,
    pub variables: Option<HashMap<String, JsonValue>>,

    #[serde(flatten)]
    pub common: Arc<CommonItemInfo>,
}

impl ItemWithCommonInfo for Tool {
    fn common(&self) -> Arc<CommonItemInfo> {
        self.common.clone()
    }
}

#[expect(clippy::doc_markdown)]
/// 'TOOL_ARMOR' in CDDA
#[derive(Debug, Deserialize)]
pub struct ToolClothing {
    pub ammo: Option<MaybeFlatVec<UntypedInfoId>>,
    pub environmental_protection_with_filter: Option<u8>,
    pub weight_capacity_bonus: Option<Mass>,
    pub charges_per_use: Option<u16>,
    pub turns_per_charge: Option<u16>,
    pub power_draw: Option<Arc<str>>,
    pub revert_to: Option<Arc<str>>,

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
    pub pocket_mods: Option<Arc<[HashMap<String, JsonValue>]>>,

    #[serde(default)]
    pub acceptable_ammo: Vec<JsonValue>,

    #[serde(default)]
    pub magazine_adaptor: Vec<JsonValue>,

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

pub enum ItemTypeDetails {
    Ammo(Arc<Ammo>),
    BionicItem(Arc<BionicItem>),
    Book(Arc<Book>),
    Clothing(Arc<Clothing>),
    Comestible(Arc<Comestible>),
    Engine(Arc<Engine>),
    GenericItem(Arc<GenericItem>),
    Gun(Arc<Gun>),
    Gunmod(Arc<Gunmod>),
    Magazine(Arc<Magazine>),
    PetArmor(Arc<PetArmor>),
    Tool(Arc<Tool>),
    ToolClothing(Arc<ToolClothing>),
    Toolmod(Arc<Toolmod>),
    Wheel(Arc<Wheel>),
    Craft,
}

// Using #[derive(Debug)] would cause a stack overflow, because of the bidirectional Arc usage
impl fmt::Debug for ItemTypeDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Ammo(_) => "Ammo",
            Self::BionicItem(_) => "BionicItem",
            Self::Book(_) => "Book",
            Self::Clothing(_) => "Clothing",
            Self::Comestible(_) => "Comestible",
            Self::Engine(_) => "Engine",
            Self::GenericItem(_) => "GenericItem",
            Self::Gun(_) => "Gun",
            Self::Gunmod(_) => "Gunmod",
            Self::Magazine(_) => "Magazine",
            Self::PetArmor(_) => "PetArmor",
            Self::Tool(_) => "Tool",
            Self::ToolClothing(_) => "ToolClothing",
            Self::Toolmod(_) => "Toolmod",
            Self::Wheel(_) => "Wheel",
            Self::Craft => "Craft",
        })
    }
}

impl ItemTypeDetails {
    #[must_use]
    pub fn ammo_type(&self) -> Option<&MaybeFlatVec<UntypedInfoId>> {
        match self {
            Self::Ammo(ammo) => Some(&ammo.ammo_type),
            Self::Magazine(magazine) => Some(&magazine.ammo_type),
            Self::ToolClothing(tool_clothing) => tool_clothing.ammo.as_ref(),
            Self::Tool(tool) => tool.ammo.as_ref(),
            _ => None,
        }
    }
}

impl From<Arc<Ammo>> for ItemTypeDetails {
    fn from(arc: Arc<Ammo>) -> Self {
        Self::Ammo(arc)
    }
}

impl From<Arc<BionicItem>> for ItemTypeDetails {
    fn from(arc: Arc<BionicItem>) -> Self {
        Self::BionicItem(arc)
    }
}

impl From<Arc<Book>> for ItemTypeDetails {
    fn from(arc: Arc<Book>) -> Self {
        Self::Book(arc)
    }
}

impl From<Arc<Clothing>> for ItemTypeDetails {
    fn from(arc: Arc<Clothing>) -> Self {
        Self::Clothing(arc)
    }
}

impl From<Arc<Comestible>> for ItemTypeDetails {
    fn from(arc: Arc<Comestible>) -> Self {
        Self::Comestible(arc)
    }
}

impl From<Arc<Engine>> for ItemTypeDetails {
    fn from(arc: Arc<Engine>) -> Self {
        Self::Engine(arc)
    }
}

impl From<Arc<GenericItem>> for ItemTypeDetails {
    fn from(arc: Arc<GenericItem>) -> Self {
        Self::GenericItem(arc)
    }
}

impl From<Arc<Gun>> for ItemTypeDetails {
    fn from(arc: Arc<Gun>) -> Self {
        Self::Gun(arc)
    }
}

impl From<Arc<Gunmod>> for ItemTypeDetails {
    fn from(arc: Arc<Gunmod>) -> Self {
        Self::Gunmod(arc)
    }
}

impl From<Arc<Magazine>> for ItemTypeDetails {
    fn from(arc: Arc<Magazine>) -> Self {
        Self::Magazine(arc)
    }
}

impl From<Arc<PetArmor>> for ItemTypeDetails {
    fn from(arc: Arc<PetArmor>) -> Self {
        Self::PetArmor(arc)
    }
}

impl From<Arc<Tool>> for ItemTypeDetails {
    fn from(arc: Arc<Tool>) -> Self {
        Self::Tool(arc)
    }
}

impl From<Arc<ToolClothing>> for ItemTypeDetails {
    fn from(arc: Arc<ToolClothing>) -> Self {
        Self::ToolClothing(arc)
    }
}

impl From<Arc<Toolmod>> for ItemTypeDetails {
    fn from(arc: Arc<Toolmod>) -> Self {
        Self::Toolmod(arc)
    }
}

impl From<Arc<Wheel>> for ItemTypeDetails {
    fn from(arc: Arc<Wheel>) -> Self {
        Self::Wheel(arc)
    }
}

#[derive(Debug, Deserialize)]
pub struct CommonItemInfo {
    pub id: InfoId<Self>,

    #[serde(skip)]
    pub type_details: OnceLock<ItemTypeDetails>,

    pub name: ItemName,
    pub symbol: char,
    pub description: Description,

    pub category: Option<Arc<str>>,

    // example: { "price": 0.7, "damage": { "damage_type": "bullet", "amount": 0.9 }, "dispersion": 1.1 }
    pub proportional: Option<JsonValue>,

    // example: { "damage": { "damage_type": "bullet", "amount": -1, "armor_penetration": 2 } }
    pub relative: Option<JsonValue>,

    pub count: Option<u32>,
    pub stack_size: Option<u8>,
    pub range: Option<i16>, // examples: -6, 140
    pub dispersion: Option<u16>,
    pub recoil: Option<u16>,
    pub loudness: Option<u16>,

    pub volume: Option<Volume>,
    pub integral_volume: Option<JsonValue>,

    #[serde(rename = "weight")]
    pub mass: Option<Mass>,

    #[serde(rename = "integral_weight")]
    pub integral_mass: Option<JsonValue>,

    pub longest_side: Option<Arc<str>>,
    pub integral_longest_side: Option<JsonValue>,

    pub price: Option<Price>,
    pub price_postapoc: Option<Price>,

    pub bashing: Option<u16>,
    pub cutting: Option<u16>,
    pub to_hit: Option<ToHit>,
    pub variant_type: Option<JsonValue>,
    pub variants: Option<JsonValue>,
    pub container: Option<Arc<str>>,

    #[serde(default)]
    pub sealed: bool,

    pub emits: Option<JsonValue>,
    pub explode_in_fire: Option<bool>,
    pub solar_efficiency: Option<JsonValue>,
    pub ascii_picture: Option<JsonValue>,
    pub thrown_damage: Option<JsonValue>,
    pub repairs_like: Option<JsonValue>,
    pub weapon_category: Option<JsonValue>,
    pub degradation_multiplier: Option<JsonValue>,

    pub color: Option<Arc<str>>,
    pub material: Option<MaybeFlatVec<Material>>,
    pub material_thickness: Option<f32>,
    pub chat_topics: Option<JsonValue>,

    #[serde(default)]
    pub phase: CddaPhase,

    pub magazines: Option<JsonValue>,
    pub min_skills: Option<JsonValue>,
    pub explosion: Option<JsonValue>,
    pub flags: Flags,
    pub faults: Option<JsonValue>,

    #[serde(default)]
    pub qualities: Vec<ItemQuality>,

    #[serde(default)]
    pub repairs_with: Vec<JsonValue>,

    // example: { "effects": [ "RECYCLED" ] }
    pub extend: Option<JsonValue>,

    // example: { "effects": [ "NEVER_MISFIRES" ], "flags": [ "IRREPLACEABLE_CONSUMABLE" ] }
    pub delete: Option<JsonValue>,

    pub properties: Option<JsonValue>,
    pub techniques: Option<JsonValue>,
    pub max_charges: Option<u16>,
    pub initial_charges: Option<u16>,

    #[serde(default)]
    pub use_action: MaybeFlatVec<UseAction>,

    pub countdown_interval: Option<JsonValue>,
    pub countdown_destroy: Option<JsonValue>,
    pub countdown_action: Option<JsonValue>,
    pub looks_like: Option<UntypedInfoId>,
    pub conditional_names: Option<JsonValue>,
    pub armor_data: Option<JsonValue>,
    pub pet_armor_data: Option<JsonValue>,
    pub gun_data: Option<JsonValue>,
    pub bionic_data: Option<JsonValue>,
    pub seed_data: Option<JsonValue>,
    pub relic_data: Option<JsonValue>,
    pub milling: Option<JsonValue>,
    pub gunmod_data: Option<JsonValue>,
    pub pocket_data: Option<Vec<Arc<PocketInfo>>>,
    pub armor: Option<Vec<JsonValue>>,
    pub snippet_category: Option<JsonValue>,

    // Plenty of fields already availalble
    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

impl CommonItemInfo {
    #[must_use]
    pub fn melee_damage(&self) -> u16 {
        self.bashing.unwrap_or(0).max(self.cutting.unwrap_or(0))
    }
}

impl PartialEq for CommonItemInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for CommonItemInfo {}

impl Hash for CommonItemInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
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

        #[serde(alias = "//NOLINT(cata-text-style)")]
        #[serde(alias = "//~")]
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

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CddaPhase {
    #[default]
    Solid,
    Liquid,
    Gas,
}

#[expect(clippy::struct_excessive_bools)]
#[derive(Debug, Deserialize)]
pub struct PocketInfo {
    #[serde(default)]
    pub pocket_type: PocketType,

    #[serde(default)]
    pub ablative: bool,

    #[serde(default)]
    pub airtight: bool,

    #[serde(default)]
    pub forbidden: bool,

    #[serde(default)]
    pub inherits_flags: bool,

    #[serde(default)]
    pub holster: bool,

    #[serde(default)]
    pub open_container: bool,

    #[serde(default)]
    pub rigid: bool,

    #[serde(default)]
    pub transparent: bool,

    #[serde(default)]
    pub watertight: bool,

    pub description: Option<Arc<str>>,

    pub min_contains_volume: Option<Volume>,
    pub max_contains_volume: Option<Volume>,
    pub max_contains_weight: Option<Mass>,
    pub min_item_length: Option<Distance>,
    pub max_item_length: Option<Distance>,
    pub min_item_volume: Option<Volume>,
    pub max_item_volume: Option<Volume>,
    pub magazine_well: Option<Volume>,

    #[serde(default)]
    pub flag_restriction: Vec<Arc<str>>,

    #[serde(default)]
    pub item_restriction: Vec<InfoId<CommonItemInfo>>,

    pub activity_noise: Option<JsonValue>,
    pub allowed_speedloaders: Option<JsonValue>,
    pub ammo_restriction: Option<HashMap<Arc<str>, u32>>,
    pub default_magazine: Option<Arc<str>>,
    pub extra_encumbrance: Option<u8>,
    pub material_restriction: Option<JsonValue>,
    pub moves: Option<u16>,
    pub ripoff: Option<u8>,
    pub sealed_data: Option<SealedData>,
    pub spoil_multiplier: Option<f32>,
    pub volume_encumber_modifier: Option<f32>,
    pub volume_multiplier: Option<f32>,
    pub weight_multiplier: Option<f32>,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize, VariantArray,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PocketType {
    // Based on item_pocket.h:40-48
    // Order-dependant!
    #[default]
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

#[derive(Debug, Deserialize)]
pub struct SealedData {
    pub spoil_multiplier: f32,

    #[serde(flatten)]
    pub ignored: Ignored<Self>,
}

#[cfg(test)]
mod item_tests {
    use super::*;
    use serde_json::from_str as from_json_str;

    #[test]
    fn ghee_works() {
        let json = include_str!("test_data/ghee.json");
        let result = from_json_str::<CommonItemInfo>(json);
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn mc_jian_works() {
        let json = include_str!("test_data/mc_jian.json");
        let result = from_json_str::<CommonItemInfo>(json);
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn mutagen_works() {
        let json = include_str!("test_data/mutagen_beast.json");
        let result = from_json_str::<CommonItemInfo>(json);
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn pocket_works() {
        let json = include_str!("test_data/pocket.json");
        let result = from_json_str::<PocketInfo>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
