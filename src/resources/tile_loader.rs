use bevy::prelude::*;
use bevy::utils::HashMap;
use rand::seq::SliceRandom;
use serde::Deserialize;
use std::fs::read_to_string;

use super::super::components::Label;

#[derive(Hash, PartialEq, Eq, Debug, Clone, Deserialize)]
pub struct TileName(pub String);

impl TileName {
    pub fn new(value: &'static str) -> Self {
        Self(value.into())
    }

    pub fn to_label(&self) -> Label {
        Label::new(self.0.clone())
    }

    fn variants(&self) -> Vec<Self> {
        let mut result = vec![Self(self.0.clone() + "_season_summer"), self.clone()];
        if let Some(index) = self.0.rfind('_') {
            result.push(Self(self.0[..index].to_string()));
        }
        result
    }
}

#[derive(Debug)]
struct TileInfo {
    names: Vec<TileName>,
    foreground: Vec<SpriteNumber>,
    background: Vec<SpriteNumber>,
}

impl TileInfo {
    fn new(tile: &serde_json::Value) -> Self {
        let tile = tile.as_object().unwrap();
        Self {
            names: {
                let mut tile_names = Vec::new();
                match &tile["id"] {
                    serde_json::Value::String(s) => {
                        tile_names.push(TileName(s.to_string()));
                    }
                    serde_json::Value::Array(list) => {
                        for item in list {
                            tile_names.push(TileName(item.as_str().unwrap().to_string()));
                        }
                    }
                    other => panic!("{other:?}"),
                };
                tile_names
            },
            foreground: load_xg(tile.get("fg")),
            background: load_xg(tile.get("bg")),
        }
    }

    fn sprite_numbers(&self) -> (Option<SpriteNumber>, Option<SpriteNumber>) {
        (
            self.foreground.choose(&mut rand::thread_rng()).copied(),
            self.background.choose(&mut rand::thread_rng()).copied(),
        )
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

#[derive(PartialEq, Eq, PartialOrd, Debug, Clone, Copy, Hash)]
pub struct SpriteNumber(usize);

impl SpriteNumber {
    fn from_json(value: &serde_json::Value) -> Self {
        Self(value.as_u64().unwrap() as usize)
    }

    fn from_number(n: &serde_json::Number) -> Self {
        Self(n.as_u64().unwrap() as usize)
    }

    fn to_sprite(self, atlas_wrapper: &AtlasWrapper) -> TextureAtlasSprite {
        assert!(
            atlas_wrapper.contains(&self),
            "{:?} {:?}",
            atlas_wrapper.from,
            &self
        );
        TextureAtlasSprite::new(self.0 - atlas_wrapper.from.0)
    }
}

#[derive(Clone, Debug)]
struct SpriteTemplate {
    texture_atlas: Handle<TextureAtlas>,
    translation: Vec3,
    sprite: TextureAtlasSprite,
}

impl SpriteTemplate {
    pub fn sprite_sheet_bundle(self, transform: Transform) -> SpriteSheetBundle {
        println!("{:?}", self.sprite.index);
        SpriteSheetBundle {
            sprite: self.sprite,
            texture_atlas: self.texture_atlas,
            transform: Transform::from_translation(self.translation) * transform,
            ..SpriteSheetBundle::default()
        }
    }
}

struct AtlasWrapper {
    from: SpriteNumber,
    to: SpriteNumber,
    handle: Handle<TextureAtlas>,
    offset: Vec3,
}

impl AtlasWrapper {
    fn new(
        texture_atlases: &mut Assets<TextureAtlas>,
        asset_server: &AssetServer,
        json: &serde_json::Value,
        tiles: &mut HashMap<TileName, TileInfo>,
    ) -> Option<Self> {
        let atlas = json.as_object().unwrap();
        let filename = atlas["file"].as_str().unwrap();
        if filename == "fallback.png" {
            return None;
        }
        let filepath = "gfx/UltimateCataclysm/".to_string() + filename;

        let from_to = if let Some(comment) = atlas.get("//") {
            comment
                .as_str()
                .unwrap()
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

        dbg!(&filepath);
        println!(
            "{:?} {:?} | {:?}-{:?} | {:?}",
            width,
            height,
            from_to[0],
            from_to[1],
            (from_to[1].0 - from_to[0].0) as usize / 16 + 1
        );
        let handle = texture_atlases.add(TextureAtlas::from_grid(
            asset_server.load(filepath.as_str()),
            Vec2::new(width, height),
            16,
            (from_to[1].0 - from_to[0].0) as usize / 16 + 1,
        ));

        let offset_x = atlas
            .get("sprite_offset_x")
            .and_then(serde_json::Value::as_i64)
            .map_or(0.0, |x| x as f32 / 32.0);
        let offset_y = atlas
            .get("sprite_offset_y")
            .and_then(serde_json::Value::as_i64)
            .map_or(0.0, |y| y as f32 / 32.0);

        // 2D -> 3D
        // upper left offset -> center offset
        let offset = Vec3::new(
            0.5 * height - 0.5 - offset_y,
            0.0,
            0.5 * width - 0.5 - offset_x,
        );

        for tile in atlas["tiles"].as_array().unwrap() {
            let tile_info = TileInfo::new(tile);
            for name in &tile_info.names {
                tiles.insert(name.clone(), tile_info.clone());
            }
        }

        Some(Self {
            from: from_to[0],
            to: from_to[1],
            handle,
            offset,
        })
    }

    fn contains(&self, sprite_number: &SpriteNumber) -> bool {
        (self.from..=self.to).contains(sprite_number)
    }
}

pub struct TileLoader {
    tiles: HashMap<TileName, TileInfo>,
    sprites: HashMap<SpriteNumber, SpriteTemplate>,
    pub atlas: Handle<TextureAtlas>, // TODO
}

impl TileLoader {
    pub fn new(texture_atlases: &mut Assets<TextureAtlas>, asset_server: &AssetServer) -> Self {
        let filepath = "assets/gfx/UltimateCataclysm/tile_config.json";
        let file_contents = read_to_string(&filepath).unwrap();
        let json: serde_json::Value = serde_json::from_str(&file_contents).unwrap();
        let atlases = json.as_object().unwrap()["tiles-new"].as_array().unwrap();

        let mut atlas_wrappers = Vec::new();
        let mut tiles = HashMap::default();

        for atlas in atlases {
            if let Some(atlas) = AtlasWrapper::new(texture_atlases, asset_server, atlas, &mut tiles)
            {
                atlas_wrappers.push(atlas);
            }
        }

        let mut loader = Self {
            tiles,
            sprites: HashMap::default(),
            atlas: atlas_wrappers[1].handle.clone(),
        };

        for tile_info in loader.tiles.values() {
            for fg in &tile_info.foreground {
                loader
                    .sprites
                    .entry(*fg)
                    .or_insert_with(|| Self::sprite_template(&atlas_wrappers, fg));
            }
            for bg in &tile_info.background {
                loader
                    .sprites
                    .entry(*bg)
                    .or_insert_with(|| Self::sprite_template(&atlas_wrappers, bg));
            }
        }

        loader
    }

    fn sprite_template(
        atlas_wrappers: &[AtlasWrapper],
        sprite_number: &SpriteNumber,
    ) -> SpriteTemplate {
        atlas_wrappers
            .iter()
            .find(|atlas_wrapper| atlas_wrapper.contains(sprite_number))
            .map(|atlas_wrapper| SpriteTemplate {
                texture_atlas: atlas_wrapper.handle.clone(),
                translation: atlas_wrapper.offset,
                sprite: sprite_number.to_sprite(atlas_wrapper),
            })
            .unwrap_or_else(|| panic!("{sprite_number:?} not found"))
    }

    pub fn sprite_sheet_bundles(
        &self,
        tile_name: &TileName,
        rotation: &Quat,
    ) -> Vec<SpriteSheetBundle> {
        let mut bundles = Vec::new();
        let (foreground, background) = tile_name
            .variants()
            .iter()
            .find_map(|variant| self.tiles.get(variant))
            .unwrap_or_else(|| self.tiles.get(&TileName::new("unknown")).unwrap())
            .sprite_numbers();
        println!("{tile_name:?} {:?} {:?}", &foreground, &background);
        bundles.extend(foreground.map(|fg| {
            self.sprites[&fg].clone().sprite_sheet_bundle(Transform {
                translation: if background.is_some() {
                    Vec3::new(0.0, 0.001, 0.0)
                } else {
                    Vec3::ZERO
                },
                rotation: *rotation,
                ..Transform::default()
            })
        }));
        bundles.extend(background.map(|bg| {
            self.sprites[&bg]
                .clone()
                .sprite_sheet_bundle(Transform::from_rotation(*rotation))
        }));
        bundles
    }

    /*pub fn debug(&self, pos: Pos, rotation: &Quat) -> Vec<SpriteSheetBundle> {
        self.sprites
            .get(&SpriteNumber((7760 + (pos.2 % 48) + 48 * pos.0) as u32))
            .cloned()
            .map(|x| x.sprite_sheet_bundle(Transform::from_rotation(*rotation)))
            .iter()
            .cloned()
            .collect()
    }*/
}

fn load_xg(xg: Option<&serde_json::Value>) -> Vec<SpriteNumber> {
    if let Some(xg) = xg {
        match xg {
            serde_json::Value::Number(n) => vec![SpriteNumber::from_number(n)],
            serde_json::Value::Array(list) => {
                let mut ids = Vec::new();
                for item in list {
                    match item {
                        serde_json::Value::Number(n) => ids.push(SpriteNumber::from_number(n)),
                        serde_json::Value::Object(obj) => {
                            ids.push(SpriteNumber::from_json(obj.get("sprite").unwrap()));
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
