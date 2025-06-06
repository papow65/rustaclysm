use bevy_log::error;
use serde::de::{Deserialize, Deserializer, Error as _, SeqAccess, Visitor};
use std::{fmt, sync::Arc};

/// A player's memory of terrain on 8x8 suzones or 4x4 zones. Corresponds to a map memory ('.mmr') file in CDDA.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum MapMemory {
    List(Vec<SubmapMemory>),
    Map {
        version: u8,
        data: Vec<SubmapMemory>,
    },
}

impl MapMemory {
    #[must_use]
    pub fn list(&self) -> &[SubmapMemory] {
        match self {
            Self::List(list) => list,
            Self::Map { data, .. } => data,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(from = "Option<Vec<TileMemory>>")]
pub struct SubmapMemory(pub Vec<TileMemory>);

impl SubmapMemory {
    #[must_use]
    pub fn seen(&self, x: u8, z: u8) -> bool {
        assert!(self.0.len() <= 144, ":-(");

        let tile = z * 12 + x;
        let mut next_tile: u8 = 0;
        for tile_memory in &self.0 {
            next_tile += tile_memory.get_amount();
            if tile < next_tile {
                return tile_memory.get_type_id().is_some();
            }
        }
        panic!("{tile} {next_tile}");
    }
}

impl Default for SubmapMemory {
    fn default() -> Self {
        Self(vec![TileMemory::Old {
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
pub enum TileMemory {
    Old {
        // Empty strings are translated to `None`.
        type_id: Option<Arc<str>>,

        subtile: u8,
        rotation: u16,
        symbol: u8,
        amount: u8,
    },
    New {
        amount: u8,
        unknown_a: u8,
        // Empty strings are translated to `None`.
        type_id: Option<Arc<str>>,
        unknown_b: u8,
        unknown_c: u8,
        furniture_id: Option<Arc<str>>,
        unknown_d: Option<u8>,
        unknown_e: Option<u8>,
    },
}

impl TileMemory {
    const fn get_amount(&self) -> u8 {
        match self {
            Self::New { amount, .. } | Self::Old { amount, .. } => *amount,
        }
    }

    const fn get_type_id(&self) -> Option<&Arc<str>> {
        match self {
            Self::New { type_id, .. } | Self::Old { type_id, .. } => type_id.as_ref(),
        }
    }
}

impl<'de> Deserialize<'de> for TileMemory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer
            .deserialize_seq(TileMemoryVisitor)
            .inspect_err(|error| {
                error!("{error:?}");
            })
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
        let first: Option<serde_json::Value> = seq.next_element()?;

        Ok(match first {
            Some(serde_json::Value::String(type_id)) => TileMemory::Old {
                type_id: if type_id.is_empty() {
                    None
                } else {
                    Some(type_id.into())
                },
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
            },
            Some(serde_json::Value::Number(amount)) => TileMemory::New {
                amount: amount
                    .as_u64()
                    .ok_or_else(|| A::Error::custom("Weird amount"))? as u8,
                unknown_a: seq
                    .next_element()?
                    .ok_or_else(|| A::Error::custom("Missing unknown field unknown_a"))?,
                type_id: seq
                    .next_element()?
                    .map(|s: Arc<str>| if s.is_empty() { None } else { Some(s) })
                    .ok_or_else(|| A::Error::custom("Missing type_id"))?,
                unknown_b: seq
                    .next_element()?
                    .ok_or_else(|| A::Error::custom("Missing unknown field unknown_b"))?,
                unknown_c: seq
                    .next_element()?
                    .ok_or_else(|| A::Error::custom("Missing unknown field unknown_c"))?,
                furniture_id: seq.next_element()?,
                unknown_d: seq.next_element()?,
                unknown_e: seq.next_element()?,
            },
            _unexpected => Err(A::Error::custom("Unexpected: {_unexpected :?}"))?,
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
                &Ok([
                    TileMemory::Old {
                        type_id: Some(ref x),
                        subtile: 0,
                        rotation: 0,
                        symbol: 0,
                        amount: 2
                    },
                    TileMemory::Old {
                        type_id: Some(ref y),
                        subtile: 3,
                        rotation: 2,
                        symbol: 0,
                        amount: 1
                    },
                    TileMemory::Old {
                        type_id: Some(ref z),
                        subtile: 5,
                        rotation: 0,
                        symbol: 0,
                        amount: 1
                    }
                ])
                if &**x == "t_grass" && &**y == "t_grass" && &**z == "f_black_eyed_susan"
            ),
            "{result:?}"
        );
    }
}
