use crate::cdda::tile::{atlas::Atlas, tile_info::TileInfo};
use crate::cdda::{Error, SpriteNumber, TextureInfo};
use crate::common::{AsyncNew, Paths};
use crate::gameplay::{Layers, Model, ObjectDefinition, ObjectId, SpriteLayer};
use bevy::prelude::Resource;
use bevy::utils::{Entry, HashMap};
use std::fs::read_to_string;

#[derive(Resource)]
pub(crate) struct TileLoader {
    tiles: HashMap<ObjectId, TileInfo>,
    textures: HashMap<SpriteNumber, TextureInfo>,
}

impl TileLoader {
    pub(crate) fn try_new() -> Result<Self, Error> {
        let file_name = "tile_config.json";
        let file_path = Paths::gfx_path().join("UltimateCataclysm").join(file_name);
        let file_contents = read_to_string(&file_path)?;
        let json: serde_json::Value =
            serde_json::from_str(&file_contents).map_err(|serde_json_error| Error::Json {
                _wrapped: serde_json_error,
                _file_path: file_path,
                _contents: file_contents,
            })?;
        let Some(json_object) = json.as_object() else {
            return Err(Error::UnexpectedJsonVariant {
                _format: file_name,
                _part: None,
                _expected: "object",
                _json: json,
            });
        };
        let Some(json_tiles) = json_object.get("tiles-new") else {
            return Err(Error::MissingJsonKey {
                _format: file_name,
                _key: "tiles-new",
                _json: json,
            });
        };
        let Some(json_atlases) = json_tiles.as_array() else {
            return Err(Error::UnexpectedJsonVariant {
                _format: file_name,
                _part: Some("tiles-new"),
                _expected: "array",
                _json: json,
            });
        };

        let mut tiles = HashMap::default();

        let atlases = json_atlases
            .iter()
            .map(|json_atlas| Atlas::try_new(json_atlas, &mut tiles))
            .collect::<Result<Vec<_>, _>>()?;

        let mut textures = HashMap::default();

        for sprite_number in tiles.values().flat_map(TileInfo::used_sprite_numbers) {
            if let Entry::Vacant(vacant) = textures.entry(sprite_number) {
                vacant.insert(Self::texture_info(&atlases, sprite_number)?);
            }
        }

        Ok(Self { tiles, textures })
    }

    fn texture_info(atlases: &[Atlas], sprite_number: SpriteNumber) -> Result<TextureInfo, Error> {
        atlases
            .iter()
            .find(|atlas| atlas.contains(sprite_number))
            .map(|atlas| atlas.texture_info(sprite_number))
            .ok_or(Error::UnknownSpriteNumber {
                _number: sprite_number,
            })
    }

    pub(crate) fn get_models(
        &self,
        definition: &ObjectDefinition,
        variants: &[ObjectId],
    ) -> Layers<Model> {
        let (foreground, background) = variants
            .iter()
            .find_map(|variant| self.tiles.get(variant))
            .unwrap_or_else(|| {
                //println!("No variant found from {variants:?}. Falling back to default sprite"); // TODO
                self.tiles
                    .get(&ObjectId::new("unknown"))
                    .expect("Tile should be found")
            })
            .sprite_numbers();
        //if tile_name.0.as_str() != "t_dirt" && !tile_name.0.starts_with("t_grass") {
        //    println!("{tile_name:?} {foreground:?} {background:?}");
        //}

        let foreground_model = self.to_model(foreground, definition, SpriteLayer::Front);
        let background_model = self.to_model(background, definition, SpriteLayer::Back);

        match (foreground_model, background_model) {
            (foreground_model, Some(background_model)) => Layers {
                base: background_model,
                overlay: foreground_model,
            },
            (Some(foreground_model), None) => Layers {
                base: foreground_model,
                overlay: None,
            },
            (None, None) => {
                panic!("No foreground or background for {definition:?} and {variants:?}");
            }
        }
    }

    fn to_model(
        &self,
        sprite_number: Option<SpriteNumber>,
        definition: &ObjectDefinition,
        layer: SpriteLayer,
    ) -> Option<Model> {
        sprite_number.map(|sprite_number| {
            Model::new(
                definition,
                layer,
                sprite_number,
                &self.textures[&sprite_number],
            )
        })
    }
}

impl AsyncNew<Self> for TileLoader {
    async fn async_new() -> Self {
        async move { Self::try_new().expect("Tiles should load") }.await
    }
}
