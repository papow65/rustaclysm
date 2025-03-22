use crate::{HashMap, MaybeFlatVec, SpriteNumber, UntypedInfoId};
use bevy_log::warn;
use either::Either;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeightedSpriteNumbers {
    sprite: MaybeFlatVec<SpriteNumber>,
    weight: u16,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum SpriteNumbers {
    MaybeFlat(MaybeFlatVec<SpriteNumber>),
    Weighted(Vec<WeightedSpriteNumbers>),
}

impl SpriteNumbers {
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

    #[must_use]
    pub fn random(&self) -> Option<SpriteNumber> {
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

impl Default for SpriteNumbers {
    fn default() -> Self {
        Self::MaybeFlat(MaybeFlatVec(Vec::new()))
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CddaBasicTile {
    #[serde(rename = "id")]
    ids: MaybeFlatVec<UntypedInfoId>,

    #[serde(rename = "fg")]
    #[serde(default)]
    foreground: SpriteNumbers,
    #[serde(rename = "bg")]
    #[serde(default)]
    background: SpriteNumbers,

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
pub struct CddaTileInfo {
    #[serde(flatten)]
    base: CddaBasicTile,

    #[serde(default)]
    additional_tiles: Vec<CddaBasicTile>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BasicTile {
    foreground: SpriteNumbers,
    background: SpriteNumbers,

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
pub struct TileInfo {
    ids: Vec<UntypedInfoId>,
    base: BasicTile,
    variants: HashMap<CddaTileVariant, BasicTile>,
}

impl TileInfo {
    pub fn ids(&self) -> impl Iterator<Item = UntypedInfoId> + '_ {
        self.ids.iter().cloned()
    }

    #[must_use]
    pub fn sprite_numbers(
        &self,
        tile_variant: &Option<CddaTileVariant>,
    ) -> (bool, &SpriteNumbers, &SpriteNumbers) {
        if self.base.multitile {
            if let Some(tile_variant) = tile_variant {
                if let Some(variant) = self.variants.get(tile_variant) {
                    return (true, &variant.foreground, &variant.background);
                } else {
                    warn!(
                        "Variant {tile_variant:?} could not be found for tile {:?}",
                        &self.ids
                    );
                }
            } else {
                warn!("No variant specified for multitile {:?}", &self.ids);
            }
        }

        (false, &self.base.foreground, &self.base.background)
    }

    pub fn used_sprite_numbers(&self) -> impl Iterator<Item = SpriteNumber> + '_ {
        self.base
            .foreground
            .iter()
            .chain(self.base.background.iter())
            .chain(
                self.variants
                    .values()
                    .flat_map(|v| v.foreground.iter().chain(v.background.iter())),
            )
    }
}

impl From<CddaTileInfo> for TileInfo {
    fn from(cdda_tile: CddaTileInfo) -> Self {
        let mut variants = HashMap::new();
        for tile_variant in cdda_tile.additional_tiles {
            for variant_id in &tile_variant.ids.0 {
                variants.insert(
                    CddaTileVariant::from(variant_id.clone()),
                    BasicTile::from(tile_variant.clone()),
                );
            }
        }

        Self {
            ids: cdda_tile.base.ids.0.clone(),
            base: BasicTile::from(cdda_tile.base),
            variants,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CddaTileVariant {
    // For vehicle parts:
    Broken,
    Open,
    // For tiles:
    Center,
    Corner,
    TConnection,
    Edge,
    End,
    Unconnected,
}

impl From<UntypedInfoId> for CddaTileVariant {
    fn from(source: UntypedInfoId) -> Self {
        match &*source.fallback_name() {
            "broken" => Self::Broken,
            "open" => Self::Open,
            "center" => Self::Center,
            "corner" => Self::Corner,
            "t_connection" => Self::TConnection,
            "edge" => Self::Edge,
            "end" | "end_piece" => Self::End,
            "unconnected" => Self::Unconnected,
            unexpected => panic!("Unexpected tile variant: {unexpected:?}"),
        }
    }
}

#[cfg(test)]
mod tile_info_tests {
    use super::*;
    #[test]
    fn it_works() {
        let json = include_str!("test_water_underground.json");
        let result = serde_json::from_str::<TileInfo>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
