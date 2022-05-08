use bevy::utils::HashMap;
use serde::de::{Deserializer, SeqAccess, Visitor};
use serde::Deserialize;
use std::fs::read_to_string;

use super::tile_loader::TileName;
use crate::components::{Pos, ZoneLevel};

// Reference: https://github.com/CleverRaven/Cataclysm-DDA/blob/master/src/savegame_json.cpp

pub fn zone_layout(zone_level: ZoneLevel) -> Option<ZoneLayout> {
    let filepath = format!(
        "assets/maps/{}.{}.{}/{}.{}.{}.map",
        zone_level.x.div_euclid(32),
        zone_level.z.div_euclid(32),
        zone_level.y,
        zone_level.x,
        zone_level.z,
        zone_level.y
    );
    //println!("Path: {filepath}");
    read_to_string(&filepath)
        .ok()
        .map(|s| ZoneLayout::new(s.as_str()))
}

#[derive(Debug)]
pub struct ZoneLayout {
    pub subzone_layouts: Vec<SubzoneLayout>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubzoneLayout {
    pub version: u64,
    pub coordinates: (i32, i32, i32),
    pub turn_last_touched: u64,
    pub temperature: i64,
    pub radiation: Vec<i64>,

    #[serde(deserialize_with = "load_terrain")]
    pub terrain: Vec<TileName>,

    #[serde(deserialize_with = "load_furniture")]
    pub furniture: Vec<At<TileName>>,

    #[serde(deserialize_with = "load_items")]
    pub items: Vec<At<Item>>,

    pub traps: Vec<serde_json::Value>,
    pub fields: Vec<serde_json::Value>,
    pub cosmetics: Vec<serde_json::Value>,
    pub spawns: Vec<Spawn>,
    pub vehicles: Vec<serde_json::Value>,
    pub partial_constructions: Vec<serde_json::Value>,
    pub computers: Option<Vec<serde_json::Value>>,
}

impl ZoneLayout {
    fn new(file_contents: &str) -> Self {
        Self {
            subzone_layouts: serde_json::from_str(file_contents).unwrap(),
        }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct Furniture {
    typeid: TileName,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub typeid: TileName,
    snip_id: Option<String>,
    pub charges: Option<u16>,
    active: Option<bool>,
    corpse: Option<String>,
    name: Option<String>,
    owner: Option<String>,
    bday: Option<i64>,
    last_temp_check: Option<u64>,
    specific_energy: Option<u64>,
    temperature: Option<u64>,
    item_vars: Option<HashMap<String, String>>,
    item_tags: Option<Vec<String>>,
    contents: Option<Container>,
    components: Option<Vec<Item>>,
    is_favorite: Option<bool>,
    relic_data: Option<serde_json::Value>,
    damaged: Option<i64>,
    current_phase: Option<u8>,
    faults: Option<Vec<String>>,
    rot: Option<i64>,
    curammo: Option<String>,
    item_counter: Option<u8>,
    variant: Option<String>,
    recipe_charges: Option<u8>,
    poison: Option<u8>,
    burnt: Option<serde_json::Value>,
    craft_data: Option<serde_json::Value>,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Container {
    contents: Vec<Pocket>,
    additional_pockets: Option<Vec<Pocket>>,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pocket {
    pocket_type: u8,
    contents: Vec<Item>,
    _sealed: bool,
    allowed: Option<bool>,
    favorite_settings: Option<serde_json::Value>,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spawn {
    pub spawn_type: TileName,
    count: i32,
    pub x: i32,
    pub z: i32,
    faction_id: i32,
    mission_id: i32,
    pub friendly: bool,
    pub name: Option<String>,
}

#[derive(Debug)]
pub struct At<T> {
    x: u8,
    y: u8,
    obj: T,
}

impl<T> At<T> {
    pub const fn get(&self, relative_pos: Pos) -> Option<&T> {
        if relative_pos.0 as u8 == self.x && relative_pos.2 as u8 == self.y {
            Some(&self.obj)
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Repetition<T> {
    obj: T,
    amount: u16,
}

fn load_terrain<'de, D>(deserializer: D) -> Result<Vec<TileName>, D::Error>
where
    D: Deserializer<'de>,
{
    struct TerrainVisitor;

    impl<'de> Visitor<'de> for TerrainVisitor {
        type Value = Vec<TileName>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str(
                "a (mixed) sequence of strings and sequences containing [tile name, amount]",
            )
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut result = Vec::new();
            while let Some(element) = seq.next_element::<serde_json::Value>()? {
                parse_repetition::<TileName>(&element, &mut result);
            }
            Ok(result)
        }
    }

    deserializer.deserialize_seq(TerrainVisitor)
}

fn load_furniture<'de, D>(deserializer: D) -> Result<Vec<At<TileName>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct FurnitureVisitor;

    impl<'de> Visitor<'de> for FurnitureVisitor {
        type Value = Vec<At<TileName>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence of sequences containing [x, y, tile name]")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut result: Vec<At<TileName>> = Vec::new();
            while let Some(element) = seq.next_element::<serde_json::Value>()? {
                match element {
                    serde_json::Value::Array(list) => {
                        result.push(At::<TileName> {
                            x: list[0].as_u64().unwrap() as u8,
                            y: list[1].as_u64().unwrap() as u8,
                            obj: TileName::new(list[2].as_str().unwrap()),
                        });
                    }
                    _ => panic!("{element:?}"),
                }
            }
            Ok(result)
        }
    }

    deserializer.deserialize_seq(FurnitureVisitor)
}

fn load_items<'de, D>(deserializer: D) -> Result<Vec<At<Item>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ItemsVisitor;

    impl<'de> Visitor<'de> for ItemsVisitor {
        type Value = Vec<At<Item>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence containing [x, y, [item, ...], x, y, [item, ...], ...]")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut result: Vec<At<Item>> = Vec::new();
            let mut x = None;
            let mut y = None;
            while let Some(element) = seq.next_element::<serde_json::Value>()? {
                match element {
                    serde_json::Value::Number(n) => {
                        if x.is_none() {
                            x = Some(n.as_u64().unwrap() as u8);
                        } else {
                            y = Some(n.as_u64().unwrap() as u8);
                        }
                    }
                    serde_json::Value::Array(list) => {
                        for element in list {
                            let mut vec = Vec::new();
                            parse_repetition(&element, &mut vec);
                            for obj in vec {
                                result.push(At::<Item> {
                                    x: x.unwrap(),
                                    y: y.unwrap(),
                                    obj,
                                });
                            }
                        }
                        x = None;
                        y = None;
                    }
                    _ => panic!("{element:?}"),
                }
            }
            Ok(result)
        }
    }

    deserializer.deserialize_seq(ItemsVisitor)
}

fn parse_repetition<T>(value: &serde_json::Value, vec: &mut Vec<T>)
where
    T: Clone,
    T: for<'de> Deserialize<'de>,
{
    match value {
        serde_json::Value::Array(_) => {
            let repetition: Repetition<T> =
                helpful_unwrap(serde_json::from_value(value.clone()), value);
            for _ in 0..repetition.amount {
                vec.push(repetition.obj.clone());
            }
        }
        _ => vec.push(helpful_unwrap(
            serde_json::from_value::<T>(value.clone()),
            value,
        )),
    }
}

fn helpful_unwrap<T, E>(result: Result<T, E>, value: &serde_json::Value) -> T
where
    E: std::fmt::Debug,
{
    match result {
        Ok(item) => item,
        Err(err) => {
            panic!("{:?}\njson: {:?}", &err, value)
        }
    }
}
