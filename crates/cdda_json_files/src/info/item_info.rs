use crate::{structure::MaybeFlatVec, Flags, HashMap, ObjectId};
use serde::Deserialize;
use std::sync::Arc;
use units::{Mass, Volume};

#[derive(Debug, Deserialize)]
pub struct ItemInfo {
    pub category: Option<Arc<str>>,

    pub effects: Option<Vec<Arc<str>>>,

    // example: { "price": 0.7, "damage": { "damage_type": "bullet", "amount": 0.9 }, "dispersion": 1.1 }
    pub proportional: Option<serde_json::Value>,

    // example: { "damage": { "damage_type": "bullet", "amount": -1, "armor_penetration": 2 } }
    pub relative: Option<serde_json::Value>,

    // example: { "damage_type": "bullet", "amount": 28, "armor_penetration": 4 }
    pub shot_spread: Option<u16>,

    // example: { "damage_type": "bullet", "amount": 28, "armor_penetration": 4 }
    pub damage: Option<serde_json::Value>,

    // example: { "damage_type": "bullet", "amount": 28, "armor_penetration": 4 }
    pub shot_damage: Option<serde_json::Value>,

    pub count: Option<u32>,
    pub projectile_count: Option<u32>,
    pub stack_size: Option<u8>,
    pub ammo_type: Option<MaybeFlatVec<Arc<str>>>,
    pub casing: Option<Arc<str>>,
    pub range: Option<i16>, // examples: -6, 140
    pub dispersion: Option<u16>,
    pub recoil: Option<u16>,
    pub loudness: Option<u16>,
    pub drop: Option<Arc<str>>,
    pub show_stats: Option<bool>,

    // The fields below are listed in load_basic_info as item_factory.cpp:3932
    #[serde(rename = "weight")]
    pub mass: Option<Mass>,

    #[serde(rename = "integral_weight")]
    pub integral_mass: Option<serde_json::Value>,

    pub volume: Option<Volume>,
    pub longest_side: Option<Arc<str>>,
    pub price: Option<Price>,
    pub price_postapoc: Option<Price>,
    pub stackable: Option<serde_json::Value>,
    pub integral_volume: Option<serde_json::Value>,
    pub integral_longest_side: Option<serde_json::Value>,
    pub bashing: Option<u16>,
    pub cutting: Option<u16>,
    pub to_hit: Option<ToHit>,
    pub variant_type: Option<serde_json::Value>,
    pub variants: Option<serde_json::Value>,
    pub container: Option<Arc<str>>,
    pub sealed: Option<bool>,
    pub min_strength: Option<serde_json::Value>,
    pub min_dexterity: Option<serde_json::Value>,
    pub min_intelligence: Option<serde_json::Value>,
    pub min_perception: Option<serde_json::Value>,
    pub emits: Option<serde_json::Value>,
    pub explode_in_fire: Option<bool>,
    pub insulation: Option<serde_json::Value>,
    pub solar_efficiency: Option<serde_json::Value>,
    pub ascii_picture: Option<serde_json::Value>,
    pub thrown_damage: Option<serde_json::Value>,
    pub repairs_like: Option<serde_json::Value>,
    pub weapon_category: Option<serde_json::Value>,
    pub damage_states: Option<serde_json::Value>,
    pub degradation_multiplier: Option<serde_json::Value>,

    #[serde(rename(deserialize = "type"))]
    #[expect(dead_code)] // TODO
    type_: Arc<str>,

    pub name: ItemName,
    pub description: Option<Description>,
    pub symbol: Option<char>,
    pub color: Option<Arc<str>>,
    pub material: Option<MaybeFlatVec<Material>>,
    pub material_thickness: Option<f32>,
    pub chat_topics: Option<serde_json::Value>,
    pub phase: Option<Arc<str>>,
    pub magazines: Option<serde_json::Value>,
    pub nanofab_template_group: Option<serde_json::Value>,
    pub template_requirements: Option<serde_json::Value>,
    pub min_skills: Option<serde_json::Value>,
    pub explosion: Option<serde_json::Value>,
    pub flags: Flags,
    pub faults: Option<serde_json::Value>,

    #[serde(default)]
    pub qualities: Vec<(ObjectId, i8)>,

    // example: { "effects": [ "RECYCLED" ] }
    pub extend: Option<serde_json::Value>,

    // example: { "effects": [ "NEVER_MISFIRES" ], "flags": [ "IRREPLACEABLE_CONSUMABLE" ] }
    pub delete: Option<serde_json::Value>,

    pub charged_qualities: Option<serde_json::Value>,
    pub properties: Option<serde_json::Value>,
    pub techniques: Option<serde_json::Value>,
    pub max_charges: Option<u16>,
    pub initial_charges: Option<u16>,
    pub use_action: Option<serde_json::Value>,
    pub countdown_interval: Option<serde_json::Value>,
    pub countdown_destroy: Option<serde_json::Value>,
    pub countdown_action: Option<serde_json::Value>,
    pub drop_action: Option<serde_json::Value>,
    pub looks_like: Option<ObjectId>,
    pub conditional_names: Option<serde_json::Value>,
    pub armor_data: Option<serde_json::Value>,
    pub pet_armor_data: Option<serde_json::Value>,
    pub book_data: Option<serde_json::Value>,
    pub gun_data: Option<serde_json::Value>,
    pub bionic_data: Option<serde_json::Value>,
    pub ammo_data: Option<serde_json::Value>,
    pub seed_data: Option<serde_json::Value>,
    pub brewable: Option<serde_json::Value>,
    pub relic_data: Option<serde_json::Value>,
    pub milling: Option<serde_json::Value>,
    pub gunmod_data: Option<serde_json::Value>,
    pub pocket_data: Option<Vec<serde_json::Value>>,
    pub armor: Option<Vec<serde_json::Value>>,
    pub snippet_category: Option<serde_json::Value>,

    // Plenty of fields already availalble
    #[expect(unused)]
    #[serde(flatten)]
    extra: HashMap<Arc<str>, serde_json::Value>,
}

impl ItemInfo {
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
        let result = serde_json::from_str::<ItemInfo>(json);
        assert!(result.is_ok(), "{result:?}");
    }
    #[test]
    fn mc_jian_works() {
        let json = include_str!("test_mc_jian.json");
        let result = serde_json::from_str::<ItemInfo>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
