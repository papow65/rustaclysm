use super::SpriteNumber;
use crate::prelude::{Error, ObjectId};
use std::any::type_name;

#[derive(Clone, Debug)]
pub(super) struct TileInfo {
    pub(super) names: Vec<ObjectId>,
    foreground: Vec<SpriteNumber>,
    background: Vec<SpriteNumber>,
}

impl TileInfo {
    pub(super) fn sprite_numbers(&self) -> (Option<SpriteNumber>, Option<SpriteNumber>) {
        (
            fastrand::choice(&self.foreground).copied(),
            fastrand::choice(&self.background).copied(),
        )
    }

    pub(super) fn used_sprite_numbers(&self) -> impl Iterator<Item = SpriteNumber> + '_ {
        self.foreground
            .iter()
            .copied()
            .chain(self.background.iter().copied())
    }
}

impl TryFrom<&serde_json::Value> for TileInfo {
    type Error = Error;

    fn try_from(tile: &serde_json::Value) -> Result<Self, Self::Error> {
        let tile = tile
            .as_object()
            .expect("JSON value should be an object (map)");
        Ok(Self {
            names: {
                let mut tile_names = Vec::new();
                match &tile["id"] {
                    serde_json::Value::String(s) => {
                        tile_names.push(ObjectId::new(s));
                    }
                    serde_json::Value::Array(list) => {
                        for item in list {
                            tile_names.push(ObjectId::new(
                                item.as_str().expect("JSON value should be a string"),
                            ));
                        }
                    }
                    other => {
                        return Err(Error::UnexpectedJsonVariant {
                            _format: type_name::<Self>(),
                            _part: Some("id"),
                            _expected: "string or array",
                            _json: other.clone(),
                        });
                    }
                };
                tile_names
            },
            foreground: load_xground(tile.get("fg")),
            background: load_xground(tile.get("bg")),
        })
    }
}

fn load_xground(xg: Option<&serde_json::Value>) -> Vec<SpriteNumber> {
    if let Some(xg) = xg {
        match xg {
            serde_json::Value::Number(n) => vec![SpriteNumber::from_number(n)],
            serde_json::Value::Array(list) => {
                let mut ids = Vec::new();
                for item in list {
                    match item {
                        serde_json::Value::Number(n) => ids.push(SpriteNumber::from_number(n)),
                        serde_json::Value::Object(obj) => {
                            ids.push(SpriteNumber::from_json(
                                obj.get("sprite").expect("'sprite' key should be present"),
                            ));
                        }
                        other => panic!("{other:?}"),
                    }
                }
                ids
            }
            other => panic!("{other:?}"),
        }
    } else {
        Vec::new()
    }
}
