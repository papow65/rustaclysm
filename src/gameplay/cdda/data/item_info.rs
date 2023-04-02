use crate::prelude::{DeflatVec, Mass, ObjectId, TextLabel, Volume};
use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct ItemInfo {
    #[allow(dead_code)] // TODO
    pub(crate) category: Option<String>,

    #[allow(dead_code)] // TODO
    pub(crate) effects: Option<Vec<String>>,

    // example: { "price": 0.7, "damage": { "damage_type": "bullet", "amount": 0.9 }, "dispersion": 1.1 }
    #[allow(dead_code)] // TODO
    pub(crate) proportional: Option<serde_json::Value>,

    // example: { "damage": { "damage_type": "bullet", "amount": -1, "armor_penetration": 2 } }
    #[allow(dead_code)] // TODO
    pub(crate) relative: Option<serde_json::Value>,

    // example: { "damage_type": "bullet", "amount": 28, "armor_penetration": 4 }
    #[allow(dead_code)] // TODO
    pub(crate) shot_spread: Option<u16>,

    // example: { "damage_type": "bullet", "amount": 28, "armor_penetration": 4 }
    #[allow(dead_code)] // TODO
    pub(crate) damage: Option<serde_json::Value>,

    // example: { "damage_type": "bullet", "amount": 28, "armor_penetration": 4 }
    #[allow(dead_code)] // TODO
    pub(crate) shot_damage: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) count: Option<u32>,

    #[allow(dead_code)] // TODO
    pub(crate) projectile_count: Option<u8>,

    #[allow(dead_code)] // TODO
    pub(crate) stack_size: Option<u8>,

    #[allow(dead_code)] // TODO
    pub(crate) ammo_type: Option<DeflatVec<String>>,

    #[allow(dead_code)] // TODO
    pub(crate) casing: Option<String>,

    #[allow(dead_code)] // TODO
    pub(crate) range: Option<i16>, // examples: -6, 140

    #[allow(dead_code)] // TODO
    pub(crate) dispersion: Option<u16>,

    #[allow(dead_code)] // TODO
    pub(crate) recoil: Option<u16>,

    #[allow(dead_code)] // TODO
    pub(crate) loudness: Option<u16>,

    #[allow(dead_code)] // TODO
    pub(crate) drop: Option<String>,

    #[allow(dead_code)] // TODO
    pub(crate) show_stats: Option<bool>,

    // The fields below are listed in load_basic_info as item_factory.cpp:3932
    #[serde(rename = "weight")]
    pub(crate) mass: Option<Mass>,

    #[serde(rename = "integral_weight")]
    #[allow(dead_code)] // TODO
    pub(crate) integral_mass: Option<serde_json::Value>,

    pub(crate) volume: Option<Volume>,

    #[allow(dead_code)] // TODO
    pub(crate) longest_side: Option<String>,

    #[allow(dead_code)] // TODO
    pub(crate) price: Option<Price>,

    #[allow(dead_code)] // TODO
    pub(crate) price_postapoc: Option<Price>,

    #[allow(dead_code)] // TODO
    pub(crate) stackable: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) integral_volume: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) integral_longest_side: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) bashing: Option<u16>,

    #[allow(dead_code)] // TODO
    pub(crate) cutting: Option<u16>,

    #[allow(dead_code)] // TODO
    pub(crate) to_hit: Option<ToHit>,

    #[allow(dead_code)] // TODO
    pub(crate) variant_type: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) variants: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) container: Option<String>,

    #[allow(dead_code)] // TODO
    pub(crate) sealed: Option<bool>,

    #[allow(dead_code)] // TODO
    pub(crate) min_strength: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) min_dexterity: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) min_intelligence: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) min_perception: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) emits: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) explode_in_fire: Option<bool>,

    #[allow(dead_code)] // TODO
    pub(crate) insulation: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    #[allow(dead_code)] // TODO
    pub(crate) solar_efficiency: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) ascii_picture: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) thrown_damage: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) repairs_like: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) weapon_category: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) damage_states: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) degradation_multiplier: Option<serde_json::Value>,

    #[serde(rename(deserialize = "type"))]
    #[allow(dead_code)] // TODO
    pub(crate) type_: String,

    pub(crate) name: ItemName,

    #[allow(dead_code)] // TODO
    pub(crate) description: Option<Description>,

    #[allow(dead_code)] // TODO
    pub(crate) symbol: Option<char>,

    #[allow(dead_code)] // TODO
    pub(crate) color: Option<String>,

    #[allow(dead_code)] // TODO
    pub(crate) material: Option<DeflatVec<Material>>,

    #[allow(dead_code)] // TODO
    pub(crate) material_thickness: Option<f32>,

    #[allow(dead_code)] // TODO
    pub(crate) chat_topics: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) phase: Option<String>,

    #[allow(dead_code)] // TODO
    pub(crate) magazines: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) nanofab_template_group: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) template_requirements: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) min_skills: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) explosion: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) flags: Option<Vec<String>>,

    #[allow(dead_code)] // TODO
    pub(crate) faults: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) qualities: Option<Vec<(String, i8)>>,

    // example: { "effects": [ "RECYCLED" ] }
    #[allow(dead_code)] // TODO
    pub(crate) extend: Option<serde_json::Value>,

    // example: { "effects": [ "NEVER_MISFIRES" ], "flags": [ "IRREPLACEABLE_CONSUMABLE" ] }
    #[allow(dead_code)] // TODO
    pub(crate) delete: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) charged_qualities: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) properties: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) techniques: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) max_charges: Option<u16>,

    #[allow(dead_code)] // TODO
    pub(crate) initial_charges: Option<u16>,

    #[allow(dead_code)] // TODO
    pub(crate) use_action: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) countdown_interval: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) countdown_destroy: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) countdown_action: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) drop_action: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) looks_like: Option<ObjectId>,

    #[allow(dead_code)] // TODO
    pub(crate) conditional_names: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) armor_data: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) pet_armor_data: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) book_data: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) gun_data: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) bionic_data: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) ammo_data: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) seed_data: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) brewable: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) relic_data: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) milling: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) gunmod_data: Option<serde_json::Value>,

    #[allow(dead_code)] // TODO
    pub(crate) pocket_data: Option<Vec<serde_json::Value>>,

    #[allow(dead_code)] // TODO
    pub(crate) armor: Option<Vec<serde_json::Value>>,

    #[allow(dead_code)] // TODO
    pub(crate) snippet_category: Option<serde_json::Value>,

    // Plenty of fields already availalble
    #[allow(unused)]
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub(crate) enum CddaItemName {
    Simple(String),
    Both {
        str_sp: String,

        #[allow(unused)]
        ctxt: Option<String>,
    },
    Split {
        str: String,
        str_pl: Option<String>,

        #[allow(unused)]
        ctxt: Option<String>,

        #[allow(unused)]
        #[serde(rename(deserialize = "//~"))]
        comment: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(from = "CddaItemName")]
pub(crate) struct ItemName {
    single: String,
    plural: String,
}

impl ItemName {
    pub(crate) fn to_label(&self, amount: usize) -> TextLabel {
        TextLabel::new(if amount == 1 {
            &self.single
        } else {
            &self.plural
        })
    }
}

impl From<CddaItemName> for ItemName {
    fn from(origin: CddaItemName) -> Self {
        match origin {
            CddaItemName::Simple(string) => ItemName {
                single: string.clone(),
                plural: string + "s",
            },
            CddaItemName::Both { str_sp, .. } => ItemName {
                single: str_sp.clone(),
                plural: str_sp,
            },
            CddaItemName::Split { str, str_pl, .. } => ItemName {
                single: str.clone(),
                plural: str_pl.unwrap_or_else(|| str.clone() + "s"),
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub(crate) enum Material {
    Simple(String),
    Complex {
        #[allow(unused)]
        #[serde(rename(deserialize = "type"))]
        type_: String,

        /// assume 1 when missing
        #[allow(unused)]
        portion: Option<u16>,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub(crate) enum Price {
    Numeric(u64),
    Text(String),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub(crate) enum ToHit {
    Simple(i16),
    Complex(HashMap<String, String>),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub(crate) enum Description {
    Simple(String),
    Complex(HashMap<String, String>),
}