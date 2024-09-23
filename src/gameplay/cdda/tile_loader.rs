use crate::common::{AsyncNew, Paths};
use crate::gameplay::cdda::{Atlas, TextureInfo};
use crate::gameplay::{Layers, Model, ObjectDefinition, SpriteLayer};
use bevy::prelude::Resource;
use bevy::utils::{Entry, HashMap};
use cdda::{CddaTileConfig, Error, ObjectId, SpriteNumber, TileInfo};
use std::fs::read_to_string;
use std::sync::Arc;

#[derive(Resource)]
pub(crate) struct TileLoader {
    tiles: HashMap<ObjectId, TileInfo>,
    textures: HashMap<SpriteNumber, TextureInfo>,
}

impl TileLoader {
    pub(crate) fn try_new() -> Result<Self, Error> {
        let tileset_path = Paths::gfx_path().join("UltimateCataclysm");
        let file_path = tileset_path.join("tile_config.json");
        let file_contents = read_to_string(&file_path)?;
        let cdda_tile_config = serde_json::from_str::<CddaTileConfig>(file_contents.as_str())
            .map_err(|e| Error::Json {
                _wrapped: e,
                _file_path: file_path,
                _contents: Arc::from(file_contents.as_str()),
            })?;

        let mut tiles = HashMap::default();

        let atlases = cdda_tile_config
            .atlases
            .into_iter()
            .map(|json_atlas| Atlas::new(&tileset_path, json_atlas, &mut tiles))
            .collect::<Vec<_>>();

        let mut textures = HashMap::default();

        for sprite_number in tiles.values().flat_map(TileInfo::used_sprite_numbers) {
            let texture_info = Self::texture_info(&atlases, sprite_number)?;
            match textures.entry(sprite_number) {
                Entry::Vacant(vacant) => {
                    vacant.insert(texture_info);
                }
                Entry::Occupied(o) => {
                    if cfg!(debug_assertions) && o.get() != &texture_info {
                        eprintln!(
                            "Multiple texture infos for {sprite_number:?}: {:?} {:?}",
                            o.get(),
                            &texture_info
                        );
                    }
                }
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

#[cfg(test)]
mod recipe_tests {
    use super::*;
    #[test]
    #[ignore] // Not a proper unit test, because the config tile may not exist
    fn it_works() {
        let json = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/gfx/UltimateCataclysm/tile_config.json"
        ));
        let result = serde_json::from_str::<CddaTileConfig>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
