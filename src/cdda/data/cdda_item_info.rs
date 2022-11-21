use crate::prelude::ObjectName;
use bevy::utils::HashMap;
use serde::Deserialize;
use std::fs::read_to_string;
use std::path::PathBuf;

/** Corresponds to an item file, under data/json/items/ . */
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CddaItemInfoList(pub(crate) Vec<CddaItemInfo>);

impl TryFrom<PathBuf> for CddaItemInfoList {
    type Error = ();
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let file_contents =
            read_to_string(&path).unwrap_or_else(|_| panic!("Could not read {}", path.display()));
        //println!("Found items file: {}", path.display());
        let mut list: Vec<CddaItemInfo> = Vec::new();
        serde_json::from_str::<Vec<serde_json::Value>>(file_contents.as_str())
            .unwrap_or_else(|e| panic!("Failed to parse file {}: {e}", path.display()))
            .into_iter()
            .filter_map(|v| {
                match v
                    .get("type")
                    .expect("element with type")
                    .as_str()
                    .expect("type is a str")
                {
                    "AMMO" | "ARMOR" | "BATTERY" | "BIONIC_ITEM" | "BOOK" | "COMESTIBLE"
                    | "ENGINE" | "GENERIC" | "GUN" | "GUNMOD" | "MAGAZINE" | "PET_ARMOR"
                    | "SPELL" | "TOOL" | "TOOLMOD" | "TOOL_ARMOR" | "WHEEL" => {
                        serde_json::from_value(v.clone()).unwrap_or_else(|e| {
                            panic!("Failed to parse element {}\n{e}\n{:?}", path.display(), &v)
                        })
                    }
                    "MIGRATION" | "ammunition_type" | "enchantment" | "fault" | "item_group" => {
                        // TODO using separate types
                        None
                    }
                    other => panic!("unknown type {other}"),
                }
            })
            .for_each(|c| list.push(c));
        Ok(Self(list))
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct CddaItemInfo {
    pub(crate) id: Option<ObjectName>,

    #[serde(rename(deserialize = "abstract"))]
    pub(crate) abstract_: Option<ObjectName>,

    #[serde(rename(deserialize = "copy-from"))]
    pub(crate) copy_from: Option<ObjectName>,

    pub(crate) category: Option<String>,
    pub(crate) effects: Option<Vec<String>>,

    // example: { "price": 0.7, "damage": { "damage_type": "bullet", "amount": 0.9 }, "dispersion": 1.1 }
    pub(crate) proportional: Option<serde_json::Value>,

    // example: { "damage": { "damage_type": "bullet", "amount": -1, "armor_penetration": 2 } }
    pub(crate) relative: Option<serde_json::Value>,

    // example: { "damage_type": "bullet", "amount": 28, "armor_penetration": 4 }
    pub(crate) shot_spread: Option<u16>,

    // example: { "damage_type": "bullet", "amount": 28, "armor_penetration": 4 }
    pub(crate) damage: Option<serde_json::Value>,

    // example: { "damage_type": "bullet", "amount": 28, "armor_penetration": 4 }
    pub(crate) shot_damage: Option<serde_json::Value>,

    pub(crate) count: Option<u32>,
    pub(crate) projectile_count: Option<u8>,
    pub(crate) stack_size: Option<u8>,
    pub(crate) ammo_type: Option<AmmoType>,
    pub(crate) casing: Option<String>,
    pub(crate) range: Option<i16>, // examples: -6, 140
    pub(crate) dispersion: Option<u16>,
    pub(crate) recoil: Option<u16>,
    pub(crate) loudness: Option<u16>,
    pub(crate) drop: Option<String>,
    pub(crate) show_stats: Option<bool>,

    // The fields below are listed in load_basic_info as item_factory.cpp:3932
    pub(crate) weight: Option<String>,

    pub(crate) integral_weight: Option<serde_json::Value>,
    pub(crate) volume: Option<String>,
    pub(crate) longest_side: Option<String>,
    pub(crate) price: Option<Price>,
    pub(crate) price_postapoc: Option<Price>,
    pub(crate) stackable: Option<serde_json::Value>,
    pub(crate) integral_volume: Option<serde_json::Value>,
    pub(crate) integral_longest_side: Option<serde_json::Value>,
    pub(crate) bashing: Option<u16>,
    pub(crate) cutting: Option<u16>,
    pub(crate) to_hit: Option<ToHit>,
    pub(crate) variant_type: Option<serde_json::Value>,
    pub(crate) variants: Option<serde_json::Value>,
    pub(crate) container: Option<String>,
    pub(crate) sealed: Option<bool>,
    pub(crate) min_strength: Option<serde_json::Value>,
    pub(crate) min_dexterity: Option<serde_json::Value>,
    pub(crate) min_intelligence: Option<serde_json::Value>,
    pub(crate) min_perception: Option<serde_json::Value>,
    pub(crate) emits: Option<serde_json::Value>,
    pub(crate) explode_in_fire: Option<bool>,
    pub(crate) insulation: Option<serde_json::Value>,
    pub(crate) solar_efficiency: Option<serde_json::Value>,
    pub(crate) ascii_picture: Option<serde_json::Value>,
    pub(crate) thrown_damage: Option<serde_json::Value>,
    pub(crate) repairs_like: Option<serde_json::Value>,
    pub(crate) weapon_category: Option<serde_json::Value>,
    pub(crate) damage_states: Option<serde_json::Value>,
    pub(crate) degradation_multiplier: Option<serde_json::Value>,

    #[serde(rename(deserialize = "type"))]
    pub(crate) type_: String,

    pub(crate) name: CddaItemName,
    pub(crate) description: Option<Description>,
    pub(crate) symbol: Option<char>,
    pub(crate) color: Option<String>,
    pub(crate) material: Option<Materials>,
    pub(crate) material_thickness: Option<f32>,
    pub(crate) chat_topics: Option<serde_json::Value>,
    pub(crate) phase: Option<String>,
    pub(crate) magazines: Option<serde_json::Value>,
    pub(crate) nanofab_template_group: Option<serde_json::Value>,
    pub(crate) template_requirements: Option<serde_json::Value>,
    pub(crate) min_skills: Option<serde_json::Value>,
    pub(crate) explosion: Option<serde_json::Value>,
    pub(crate) flags: Option<Vec<String>>,
    pub(crate) faults: Option<serde_json::Value>,
    pub(crate) qualities: Option<Vec<(String, i8)>>,

    // example: { "effects": [ "RECYCLED" ] }
    pub(crate) extend: Option<serde_json::Value>,

    // example: { "effects": [ "NEVER_MISFIRES" ], "flags": [ "IRREPLACEABLE_CONSUMABLE" ] }
    pub(crate) delete: Option<serde_json::Value>,

    pub(crate) charged_qualities: Option<serde_json::Value>,
    pub(crate) properties: Option<serde_json::Value>,
    pub(crate) techniques: Option<serde_json::Value>,
    pub(crate) max_charges: Option<u16>,
    pub(crate) initial_charges: Option<u16>,
    pub(crate) use_action: Option<serde_json::Value>,
    pub(crate) countdown_interval: Option<serde_json::Value>,
    pub(crate) countdown_destroy: Option<serde_json::Value>,
    pub(crate) countdown_action: Option<serde_json::Value>,
    pub(crate) drop_action: Option<serde_json::Value>,
    pub(crate) looks_like: Option<ObjectName>,
    pub(crate) conditional_names: Option<serde_json::Value>,
    pub(crate) armor_data: Option<serde_json::Value>,
    pub(crate) pet_armor_data: Option<serde_json::Value>,
    pub(crate) book_data: Option<serde_json::Value>,
    pub(crate) gun_data: Option<serde_json::Value>,
    pub(crate) bionic_data: Option<serde_json::Value>,
    pub(crate) ammo_data: Option<serde_json::Value>,
    pub(crate) seed_data: Option<serde_json::Value>,
    pub(crate) brewable: Option<serde_json::Value>,
    pub(crate) relic_data: Option<serde_json::Value>,
    pub(crate) milling: Option<serde_json::Value>,
    pub(crate) gunmod_data: Option<serde_json::Value>,
    pub(crate) pocket_data: Option<Vec<serde_json::Value>>,
    pub(crate) armor: Option<Vec<serde_json::Value>>,
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

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub(crate) enum Materials {
    Single(Material),
    Multi(Vec<Material>),
}

impl Materials {
    pub(crate) fn to_vec(&self) -> Vec<Material> {
        match self {
            Self::Single(material) => vec![material.clone()],
            Self::Multi(material) => material.clone(),
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
        portion: Option<u8>,
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

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub(crate) enum AmmoType {
    Sinlge(String),
    Multi(Vec<String>),
}
