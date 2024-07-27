use crate::prelude::*;
use bevy::{
    prelude::*,
    utils::{Entry, HashMap},
};
use std::{any::type_name, fs::read_to_string, path::PathBuf};

#[derive(Debug)]
struct TileInfo {
    names: Vec<ObjectId>,
    foreground: Vec<SpriteNumber>,
    background: Vec<SpriteNumber>,
}

impl TileInfo {
    fn sprite_numbers(&self) -> (Option<SpriteNumber>, Option<SpriteNumber>) {
        (
            fastrand::choice(&self.foreground).copied(),
            fastrand::choice(&self.background).copied(),
        )
    }

    fn used_sprite_numbers(&self) -> impl Iterator<Item = SpriteNumber> + '_ {
        self.foreground
            .iter()
            .copied()
            .chain(self.background.iter().copied())
    }
}

impl Clone for TileInfo {
    fn clone(&self) -> Self {
        Self {
            names: self.names.clone(),
            foreground: self.foreground.clone(),
            background: self.background.clone(),
        }
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

#[derive(PartialEq, Eq, PartialOrd, Debug, Clone, Copy, Hash)]
pub(crate) struct SpriteNumber(usize);

impl SpriteNumber {
    fn from_json(value: &serde_json::Value) -> Self {
        Self(
            value
                .as_u64()
                .expect("JSON value should be an integer (>= 0)") as usize,
        )
    }

    fn from_number(n: &serde_json::Number) -> Self {
        Self(n.as_u64().expect("JSON value should be an integer (>= 0)") as usize)
    }

    pub(crate) const fn to_usize(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TextureInfo {
    pub(crate) mesh_info: MeshInfo,
    pub(crate) image_path: PathBuf,
    pub(crate) transform2d: Transform2d,
}

#[derive(Debug)]
struct Atlas {
    range: (SpriteNumber, SpriteNumber),
    image_path: PathBuf,
    transform2d: Transform2d,
}

impl Atlas {
    fn new(
        json: &serde_json::Value,
        tiles: &mut HashMap<ObjectId, TileInfo>,
    ) -> Result<Option<Self>, Error> {
        let atlas = json
            .as_object()
            .expect("JSON value should be an object (map)");
        let filename = atlas["file"]
            .as_str()
            .expect("'file' key should be present");
        if filename == "fallback.png" {
            return Ok(None);
        }
        let image_path = Paths::gfx_path().join("UltimateCataclysm").join(filename);

        let from_to = if let Some(comment) = atlas.get("//") {
            comment
                .as_str()
                .expect("Comment should be a string")
                .split(' ')
                .flat_map(str::parse)
                .map(SpriteNumber)
                .collect::<Vec<SpriteNumber>>()
        } else {
            vec![SpriteNumber(0), SpriteNumber(1024)]
        };

        let width = atlas
            .get("sprite_width")
            .and_then(serde_json::Value::as_i64)
            .map_or(1.0, |w| w as f32 / 32.0);
        let height = atlas
            .get("sprite_height")
            .and_then(serde_json::Value::as_i64)
            .map_or(1.0, |h| h as f32 / 32.0);

        let offset_x = atlas
            .get("sprite_offset_x")
            .and_then(serde_json::Value::as_f64)
            .map_or(0.0, |x| x as f32 / 32.0)
            + (0.5 * width - 0.5);
        let offset_y = -(atlas // notice the minus sign
            .get("sprite_offset_y")
            .and_then(serde_json::Value::as_f64)
            .map_or(0.0, |y| y as f32 / 32.0)
            + (0.5 * height - 0.5));

        for tile in atlas["tiles"]
            .as_array()
            .expect("'tiles' key should be present")
        {
            let tile_info = TileInfo::try_from(tile)?;
            for name in &tile_info.names {
                tiles.insert(name.clone(), tile_info.clone());
            }
        }

        Ok(Some(Self {
            range: (from_to[0], from_to[1]),
            image_path,
            transform2d: Transform2d {
                scale: Vec2::new(width, height),
                offset: Vec2::new(offset_x, offset_y),
            },
        }))
    }

    fn contains(&self, sprite_number: SpriteNumber) -> bool {
        (self.range.0..=self.range.1).contains(&sprite_number)
    }

    fn texture_info(&self, sprite_number: SpriteNumber) -> TextureInfo {
        TextureInfo {
            mesh_info: MeshInfo::new(
                sprite_number.to_usize() - self.range.0.to_usize(),
                match self.image_path.display().to_string() {
                    p if p.ends_with("filler_tall.png") => 2,
                    p if p.ends_with("large_ridden.png") => 3,
                    p if p.ends_with("giant.png") => 4,
                    p if p.ends_with("huge.png") => 4,
                    p if p.ends_with("large.png") => 8,
                    p if p.ends_with("centered.png") => 12,
                    p if p.ends_with("small.png") => 12,
                    _ => 16,
                },
                1 + self.range.1.to_usize() - self.range.0.to_usize(),
            ),
            image_path: self.image_path.clone(),
            transform2d: self.transform2d.clone(),
        }
    }
}

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
        let Some(tiles) = json_object.get("tiles-new") else {
            return Err(Error::MissingJsonKey {
                _format: file_name,
                _key: "tiles-new",
                _json: json,
            });
        };
        let Some(json_atlases) = tiles.as_array() else {
            return Err(Error::UnexpectedJsonVariant {
                _format: file_name,
                _part: Some("tiles-new"),
                _expected: "array",
                _json: json,
            });
        };

        let mut atlases = Vec::new();
        let mut tiles = HashMap::default();

        for json_atlas in json_atlases {
            if let Some(atlas) = Atlas::new(json_atlas, &mut tiles)? {
                //dbg!(&atlas);
                atlases.push(atlas);
            }
        }

        let mut loader = Self {
            tiles,
            textures: HashMap::default(),
        };

        for tile_info in loader.tiles.values() {
            //dbg!(tile_info.names[0].0.as_str());
            for sprite_number in tile_info.used_sprite_numbers() {
                if let Entry::Vacant(vacant) = loader.textures.entry(sprite_number) {
                    vacant.insert(Self::texture_info(&atlases, sprite_number)?);
                }
            }
        }

        Ok(loader)
    }

    fn texture_info(atlases: &[Atlas], sprite_number: SpriteNumber) -> Result<TextureInfo, Error> {
        atlases
            .iter()
            .find(|atlas_wrapper| atlas_wrapper.contains(sprite_number))
            .map(|atlas_wrapper| atlas_wrapper.texture_info(sprite_number))
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
