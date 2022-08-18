use crate::prelude::{ObjectName, Pos, Repetition, RepetitionBlock, ZoneLevel};
use bevy::utils::HashMap;
use serde::de::{Deserializer, SeqAccess, Visitor};
use serde::Deserialize;
use std::{fmt, fs::read_to_string};

// Reference: https://github.com/CleverRaven/Cataclysm-DDA/blob/master/src/savegame_json.cpp

/** Corresponds to a 'map' in CDDA. It defines the layout of a `ZoneLevel`. */
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Map(pub Vec<Submap>);

impl TryFrom<ZoneLevel> for Map {
    type Error = ();
    fn try_from(zone_level: ZoneLevel) -> Result<Self, ()> {
        let filepath = format!(
            "assets/save/maps/{}.{}.{}/{}.{}.{}.map",
            zone_level.x.div_euclid(32),
            zone_level.z.div_euclid(32),
            zone_level.level.h,
            zone_level.x,
            zone_level.z,
            zone_level.level.h
        );
        read_to_string(&filepath)
            .ok()
            .map(|s| {
                println!("Found map: {filepath}");
                s
            })
            .map(|s| serde_json::from_str::<Self>(s.as_str()).unwrap())
            .ok_or(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Submap {
    pub version: u64,
    pub coordinates: (i32, i32, i32),
    pub turn_last_touched: u64,
    pub temperature: i64,
    pub radiation: Vec<i64>,
    pub terrain: RepetitionBlock<ObjectName>,

    #[serde(deserialize_with = "load_at_tile_name")]
    pub furniture: Vec<At<ObjectName>>,

    #[serde(deserialize_with = "load_items")]
    pub items: Vec<At<Vec<Repetition<CddaItem>>>>,

    #[serde(deserialize_with = "load_at_tile_name")]
    pub traps: Vec<At<ObjectName>>,

    #[serde(deserialize_with = "load_at_field")]
    pub fields: Vec<At<Field>>,
    pub cosmetics: Vec<(u8, u8, String, String)>,
    pub spawns: Vec<Spawn>,
    pub vehicles: Vec<serde_json::Value>, // grep -orIE 'vehicles":\[[^]]+.{80}'  assets/save/maps/ | less
    pub partial_constructions: Vec<serde_json::Value>,
    pub computers: Option<Vec<serde_json::Value>>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct Furniture {
    tile_name: ObjectName,
}

#[allow(unused)]
#[derive(Debug)]
pub struct Field {
    pub tile_name: ObjectName,
    intensity: i32,
    age: u64,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CddaItem {
    pub typeid: ObjectName,
    snip_id: Option<String>,
    pub charges: Option<u32>,
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
    contents: Option<CddaContainer>,
    components: Option<Vec<CddaItem>>,
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
pub struct CddaContainer {
    contents: Vec<Pocket>,
    additional_pockets: Option<Vec<Pocket>>,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pocket {
    pocket_type: u8,
    contents: Vec<CddaItem>,
    _sealed: bool,
    allowed: Option<bool>,
    favorite_settings: Option<serde_json::Value>,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spawn {
    pub spawn_type: ObjectName,
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
        if relative_pos.x as u8 == self.x && relative_pos.z as u8 == self.y {
            Some(&self.obj)
        } else {
            None
        }
    }
}

fn load_at_tile_name<'de, D>(deserializer: D) -> Result<Vec<At<ObjectName>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(AtVisitor::<ObjectName>::new())
}

fn load_at_field<'de, D>(deserializer: D) -> Result<Vec<At<Field>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(AtSeqVisitor::<Field>::new())
}

fn load_items<'de, D>(deserializer: D) -> Result<Vec<At<Vec<Repetition<CddaItem>>>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(AtSeqVisitor::<Vec<Repetition<CddaItem>>>::new())
}

trait JsonLoad {
    fn load(json: &serde_json::Value) -> Self;
}

impl JsonLoad for Field {
    fn load(json: &serde_json::Value) -> Self {
        let list = json.as_array().unwrap();
        Self {
            tile_name: ObjectName::load(&list[0]),
            intensity: list[1].as_i64().unwrap() as i32,
            age: list[2].as_u64().unwrap(),
        }
    }
}

impl JsonLoad for ObjectName {
    fn load(json: &serde_json::Value) -> Self {
        Self::new(json.as_str().unwrap())
    }
}

impl JsonLoad for Vec<Repetition<CddaItem>> {
    fn load(json: &serde_json::Value) -> Self {
        let mut vec = Self::new();
        for element in json.as_array().unwrap() {
            vec.push(Repetition::try_from(element.clone()).unwrap());
        }
        vec
    }
}

struct AtVisitor<T>(std::marker::PhantomData<T>)
where
    T: JsonLoad + fmt::Debug;

impl<T: JsonLoad + fmt::Debug> AtVisitor<T> {
    const fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<'de, T: JsonLoad + fmt::Debug> Visitor<'de> for AtVisitor<T> {
    type Value = Vec<At<T>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence of sequences containing [x, y, tile name]")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut result: Vec<At<T>> = Vec::new();
        while let Some(element) = seq.next_element::<serde_json::Value>()? {
            match element {
                serde_json::Value::Array(list) => {
                    result.push(At::<T> {
                        x: list[0].as_u64().unwrap() as u8,
                        y: list[1].as_u64().unwrap() as u8,
                        obj: T::load(&list[2]),
                    });
                }
                _ => panic!("{result:?} - {element:?}"),
            }
        }
        Ok(result)
    }
}

struct AtSeqVisitor<T>(std::marker::PhantomData<T>)
where
    T: JsonLoad + fmt::Debug;

impl<T: JsonLoad + fmt::Debug> AtSeqVisitor<T> {
    const fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<'de, T: JsonLoad + fmt::Debug> Visitor<'de> for AtSeqVisitor<T> {
    type Value = Vec<At<T>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence containing [x, y, [item, ...], x, y, [item, ...], ...]")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut result: Vec<At<T>> = Vec::new();
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
                element @ serde_json::Value::Array(_) => {
                    result.push(At::<T> {
                        x: x.unwrap(),
                        y: y.unwrap(),
                        obj: T::load(&element),
                    });
                    x = None;
                    y = None;
                }
                _ => panic!("{element:?}"),
            }
        }
        Ok(result)
    }
}
