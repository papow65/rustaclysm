use crate::cdda::{MaybeFlatVec, SpriteNumber};
use crate::gameplay::ObjectId;
use bevy::utils::HashMap;
use either::Either;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WeightedSprinteNumbers {
    sprite: MaybeFlatVec<SpriteNumber>,
    weight: u16,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum SprinteNumbers {
    MaybeFlat(MaybeFlatVec<SpriteNumber>),
    Weighted(Vec<WeightedSprinteNumbers>),
}

impl SprinteNumbers {
    fn iter(&self) -> impl Iterator<Item = SpriteNumber> + '_ {
        match self {
            Self::MaybeFlat(m) => Either::Left(m.0.iter().copied()),
            Self::Weighted(w) => Either::Right(
                w.iter()
                    .flat_map(|weighted| weighted.sprite.0.iter().copied()),
            ),
        }
        .into_iter()
    }

    fn random(&self) -> Option<SpriteNumber> {
        match self {
            Self::MaybeFlat(m) => fastrand::choice(m.0.iter()).copied(),
            Self::Weighted(w) => {
                let mut choices = Vec::new();
                for numbers in w {
                    for _ in 0..numbers.weight {
                        for sprite in &numbers.sprite.0 {
                            choices.push(*sprite);
                        }
                    }
                }
                fastrand::choice(choices.iter()).copied()
            }
        }
    }
}

impl Default for SprinteNumbers {
    fn default() -> Self {
        Self::MaybeFlat(MaybeFlatVec(Vec::new()))
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CddaBasicTile {
    #[serde(rename = "id")]
    ids: MaybeFlatVec<ObjectId>,

    #[serde(rename = "fg")]
    #[serde(default)]
    foreground: SprinteNumbers,
    #[serde(rename = "bg")]
    #[serde(default)]
    background: SprinteNumbers,

    #[serde(default)] // false
    multitile: bool,
    #[serde(default)] // false
    rotates: bool,
    #[serde(default)] // false
    animated: bool,

    #[expect(unused)]
    #[serde(rename = "//")]
    comment: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CddaTileInfo {
    #[serde(flatten)]
    base: CddaBasicTile,

    #[serde(default)]
    additional_tiles: Vec<CddaBasicTile>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BasicTile {
    foreground: SprinteNumbers,
    background: SprinteNumbers,

    #[expect(unused)]
    multitile: bool,
    #[expect(unused)]
    rotates: bool,
    #[expect(unused)]
    animated: bool,
}

impl From<CddaBasicTile> for BasicTile {
    fn from(cdda_base: CddaBasicTile) -> Self {
        Self {
            foreground: cdda_base.foreground,
            background: cdda_base.background,
            multitile: cdda_base.multitile,
            rotates: cdda_base.rotates,
            animated: cdda_base.animated,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(from = "CddaTileInfo")]
pub(super) struct TileInfo {
    ids: Vec<ObjectId>,
    base: BasicTile,

    #[expect(unused)]
    variants: HashMap<ObjectId, BasicTile>,
}

impl TileInfo {
    pub(super) fn ids(&self) -> impl Iterator<Item = &ObjectId> {
        self.ids.iter()
    }

    pub(super) fn sprite_numbers(&self) -> (Option<SpriteNumber>, Option<SpriteNumber>) {
        (self.base.foreground.random(), self.base.background.random())
    }

    pub(super) fn used_sprite_numbers(&self) -> impl Iterator<Item = SpriteNumber> + '_ {
        self.base
            .foreground
            .iter()
            .chain(self.base.background.iter())
    }
}

impl From<CddaTileInfo> for TileInfo {
    fn from(cdda_tile: CddaTileInfo) -> Self {
        let mut variants = HashMap::new();
        for tile_variant in cdda_tile.additional_tiles {
            for variant_id in &tile_variant.ids.0 {
                variants.insert(variant_id.clone(), BasicTile::from(tile_variant.clone()));
            }
        }

        Self {
            ids: cdda_tile.base.ids.0.clone(),
            base: BasicTile::from(cdda_tile.base),
            variants,
        }
    }
}

#[cfg(test)]
mod recipe_tests {
    use super::*;
    #[test]
    fn it_works() {
        let json = include_str!("test_water_underground.json");
        let result = serde_json::from_str::<TileInfo>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
