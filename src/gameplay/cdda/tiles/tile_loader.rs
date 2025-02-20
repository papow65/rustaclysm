use crate::gameplay::cdda::{Atlas, TextureInfo, error::Error};
use crate::gameplay::{Layers, Model, ObjectDefinition, SpriteLayer, TileVariant};
use crate::util::{AssetPaths, AsyncNew};
use bevy::prelude::Resource;
use bevy::utils::{Entry, HashMap};
use cdda_json_files::{
    CddaTileConfig, CddaTileVariant, MaybeFlatVec, ObjectId, SpriteNumber, SpriteNumbers, TileInfo,
};
use std::{fs::read_to_string, sync::Arc};

#[derive(Resource)]
pub(crate) struct TileLoader {
    tiles: HashMap<ObjectId, Arc<TileInfo>>,
    textures: HashMap<SpriteNumber, TextureInfo>,
}

impl TileLoader {
    pub(crate) fn try_new() -> Result<Self, Error> {
        let tileset_path = AssetPaths::gfx().join("UltimateCataclysm");
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

        for sprite_number in tiles
            .values()
            .flat_map(|tile_info| tile_info.used_sprite_numbers())
        {
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
        id_variants: &[ObjectId],
        tile_variant: Option<TileVariant>,
    ) -> Layers<Model> {
        let cdda_tile_variant: Option<CddaTileVariant> = tile_variant.map(Into::into);

        let (multitile, foregrounds, backgrounds) = id_variants
            .iter()
            .find_map(|variant| self.tiles.get(variant))
            .unwrap_or_else(|| {
                //println!("No variant found from {variants:?}. Falling back to default sprite"); // TODO
                self.tiles
                    .get(&ObjectId::new("unknown"))
                    .expect("Tile should be found")
            })
            .sprite_numbers(&cdda_tile_variant);
        //if tile_name.0.as_str() != "t_dirt" && !tile_name.0.starts_with("t_grass") {
        //    println!("{tile_name:?} {foreground:?} {background:?}");
        //}

        let foreground = if let (true, Some(tile_variant), SpriteNumbers::MaybeFlat(MaybeFlatVec(vec))) =
            (multitile, tile_variant, foregrounds)
        {
            if let Some(expected_legth) = tile_variant.expected_length() {
                if vec.len() != expected_legth {
                    eprintln!(
                        "Expected {expected_legth} variants for {tile_variant:?} tiles of {definition:?}, but got {:?}",
                        &vec
                    );
                }
            }

            tile_variant.index().and_then(|index| vec.get(index))
        } else {
            None
        }
        .copied()
        .or_else(|| foregrounds.random());
        let background = backgrounds.random();

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
                panic!("No foreground or background for {definition:?} and {id_variants:?}");
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
