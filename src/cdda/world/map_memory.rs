use crate::prelude::*;
use bevy::{
    asset::{io::Reader, Asset, AssetLoader, BoxedFuture, LoadContext},
    reflect::TypePath,
};
use either::Either;
use futures_lite::AsyncReadExt;
use serde::de::{Deserialize, Deserializer, Error, SeqAccess, Visitor};
use std::fmt;

pub(crate) type MapMemoryPath = PathFor<MapMemory>;

impl MapMemoryPath {
    pub(crate) fn new(sav_path: &SavPath, zone_level: ZoneLevel) -> Self {
        let mut seen_path = sav_path.0.clone();
        seen_path.set_extension("mm1");
        let seen_path = seen_path.join(format!(
            "{}.{}.{}.mmr",
            zone_level.zone.x.div_euclid(4),
            zone_level.zone.z.div_euclid(4),
            zone_level.level.h
        ));
        Self::init(seen_path)
    }
}

/// A player's memory of terrain on 8x8 suzones or 4x4 zones. Corresponds to a map memory ('.mmr') file in CDDA.
#[derive(Debug, serde::Deserialize, Asset, TypePath)]
#[serde(deny_unknown_fields)]
pub(crate) struct MapMemory(pub(crate) Vec<SubmapMemory>);

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(from = "Option<Vec<TileMemory>>")]
pub(crate) struct SubmapMemory(pub(crate) Vec<TileMemory>);

impl SubmapMemory {
    pub(crate) fn seen(&self, x: u8, z: u8) -> bool {
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
pub(crate) struct TileMemory {
    // Empty strings are translated to `None`.
    pub(crate) type_id: Option<String>,

    #[allow(unused)]
    pub(crate) subtile: u8,

    #[allow(unused)]
    pub(crate) rotation: u16,

    #[allow(unused)]
    pub(crate) symbol: u8,

    pub(crate) amount: u8,
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
                .ok_or(A::Error::custom(String::from("Missing type_id")))?,
            subtile: seq
                .next_element()?
                .ok_or(A::Error::custom(String::from("Missing subtile")))?,
            rotation: seq
                .next_element()?
                .ok_or(A::Error::custom(String::from("Missing rotation")))?,
            symbol: seq
                .next_element()?
                .ok_or(A::Error::custom(String::from("Missing symbol")))?,
            amount: seq.next_element()?.unwrap_or(1),
        })
    }
}

#[derive(Default)]
pub(crate) struct MapMemoryLoader;

impl AssetLoader for MapMemoryLoader {
    type Asset = MapMemory;
    type Settings = ();
    type Error = Either<std::io::Error, serde_json::Error>;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader
                .read_to_end(&mut bytes)
                .await
                .inspect_err(|e| {
                    eprintln!("Map file loading error: {:?} {e:?}", load_context.path(),);
                })
                .map_err(Either::Left)?;

            let map_memory = serde_json::from_slice::<MapMemory>(&bytes)
                .map_err(|e| {
                    eprintln!(
                        "Map json loading error: {:?} {:?} {e:?}",
                        load_context.path(),
                        std::str::from_utf8(&bytes[0..40])
                    );
                    e
                })
                .map_err(Either::Right)?;
            Ok(map_memory)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mmr"]
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
