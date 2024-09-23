use bevy::{asset::Asset, reflect::TypePath};
use serde::de::{Deserialize, Deserializer, Error, SeqAccess, Visitor};
use std::fmt;

/// A player's memory of terrain on 8x8 suzones or 4x4 zones. Corresponds to a map memory ('.mmr') file in CDDA.
#[derive(Debug, serde::Deserialize, Asset, TypePath)]
#[serde(deny_unknown_fields)]
pub struct MapMemory(pub Vec<SubmapMemory>);

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(from = "Option<Vec<TileMemory>>")]
pub struct SubmapMemory(pub Vec<TileMemory>);

impl SubmapMemory {
    pub fn seen(&self, x: u8, z: u8) -> bool {
        assert!(self.0.len() <= 144, ":-(");

        let tile = z * 12 + x;
        let mut next_tile: u8 = 0;
        for tile_memory in &self.0 {
            next_tile += tile_memory.amount;
            if tile < next_tile {
                return tile_memory.type_id.is_some();
            };
        }
        panic!("{tile} {next_tile}");
    }
}

impl Default for SubmapMemory {
    fn default() -> Self {
        Self(vec![TileMemory {
            type_id: None,
            subtile: 0,
            rotation: 0,
            symbol: 0,
            amount: 144,
        }])
    }
}

impl From<Option<Vec<TileMemory>>> for SubmapMemory {
    fn from(value: Option<Vec<TileMemory>>) -> Self {
        if let Some(vec) = value {
            Self(vec)
        } else {
            Self::default()
        }
    }
}

#[derive(Debug)]
pub struct TileMemory {
    // Empty strings are translated to `None`.
    pub type_id: Option<String>,

    #[allow(unused)]
    pub subtile: u8,

    #[allow(unused)]
    pub rotation: u16,

    #[allow(unused)]
    pub symbol: u8,

    pub amount: u8,
}

impl<'de> Deserialize<'de> for TileMemory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(TileMemoryVisitor)
    }
}

struct TileMemoryVisitor;

impl<'de> Visitor<'de> for TileMemoryVisitor {
    type Value = TileMemory;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an array with one string followed by three or four integers")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        Ok(TileMemory {
            type_id: seq
                .next_element()?
                .map(|s: String| if s.is_empty() { None } else { Some(s) })
                .ok_or_else(|| A::Error::custom("Missing type_id"))?,
            subtile: seq
                .next_element()?
                .ok_or_else(|| A::Error::custom("Missing subtile"))?,
            rotation: seq
                .next_element()?
                .ok_or_else(|| A::Error::custom("Missing rotation"))?,
            symbol: seq
                .next_element()?
                .ok_or_else(|| A::Error::custom("Missing symbol"))?,
            amount: seq.next_element()?.unwrap_or(1),
        })
    }
}

#[cfg(test)]
mod container_tests {
    use super::*;
    #[test]
    fn it_works() {
        let json = include_str!("test_tile_memory.json");
        let result = serde_json::from_str::<[TileMemory; 3]>(json);
        assert!(
            matches!(
                &result,
                Ok([
                    TileMemory {
                        type_id: Some(ref x),
                        subtile: 0,
                        rotation: 0,
                        symbol: 0,
                        amount: 2
                    },
                    TileMemory {
                        type_id: Some(ref y),
                        subtile: 3,
                        rotation: 2,
                        symbol: 0,
                        amount: 1
                    },
                    TileMemory {
                        type_id: Some(ref z),
                        subtile: 5,
                        rotation: 0,
                        symbol: 0,
                        amount: 1
                    }
                ])
                if x == "t_grass" && y == "t_grass" && z == "f_black_eyed_susan"
            ),
            "{result:?}"
        );
    }
}
