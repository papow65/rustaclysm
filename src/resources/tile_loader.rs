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
            foreground: load_xground(tile.get("fg")),
            background: load_xground(tile.get("bg")),
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

    pub const fn to_usize(self) -> usize {
        self.0
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub struct MeshInfo {
    pub index: usize,
    pub width: usize,
    pub size: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum SpriteOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub enum SpriteLayer {
    Front,
    Back,
}

#[derive(Debug, Clone)]
pub struct SpriteInfo {
    pub mesh_info: MeshInfo,
    pub imagepath: String,
    pub orientation: SpriteOrientation,
    pub layer: SpriteLayer,
    pub scale: (f32, f32),
    pub offset: (f32, f32),
}

#[derive(Debug)]
struct AtlasWrapper {
    range: (SpriteNumber, SpriteNumber),
    imagepath: String,
    scale: (f32, f32),
    offset: (f32, f32),
}

impl AtlasWrapper {
    fn new(
        _asset_server: &AssetServer,
        json: &serde_json::Value,
        tiles: &mut HashMap<TileName, TileInfo>,
    ) -> Option<Self> {
        let atlas = json.as_object().unwrap();
        let filename = atlas["file"].as_str().unwrap();
        if filename == "fallback.png" {
            return None;
        }
        let imagepath = "gfx/UltimateCataclysm/".to_string() + filename;

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

        for tile in atlas["tiles"].as_array().unwrap() {
            let tile_info = TileInfo::new(tile);
            for name in &tile_info.names {
                tiles.insert(name.clone(), tile_info.clone());
            }
        }

        Some(Self {
            range: (from_to[0], from_to[1]),
            imagepath,
            scale: (width, height),
            offset: (offset_x, offset_y),
        })
    }

    fn contains(&self, sprite_number: &SpriteNumber) -> bool {
        (self.range.0..=self.range.1).contains(sprite_number)
    }
}

pub struct TileLoader {
    tiles: HashMap<TileName, TileInfo>,
    sprites: HashMap<SpriteNumber, SpriteInfo>,
}

impl TileLoader {
    pub fn new(asset_server: &AssetServer) -> Self {
        let filepath = "assets/gfx/UltimateCataclysm/tile_config.json";
        let file_contents = read_to_string(&filepath).unwrap();
        let json: serde_json::Value = serde_json::from_str(&file_contents).unwrap();
        let atlases = json.as_object().unwrap()["tiles-new"].as_array().unwrap();

        let mut atlas_wrappers = Vec::new();
        let mut tiles = HashMap::default();

        for atlas in atlases {
            if let Some(atlas) = AtlasWrapper::new(asset_server, atlas, &mut tiles) {
                dbg!(&atlas);
                atlas_wrappers.push(atlas);
            }
        }

        let mut loader = Self {
            tiles,
            sprites: HashMap::default(),
        };

        for tile_info in loader.tiles.values() {
            dbg!(tile_info.names[0].0.as_str());
            for fg in &tile_info.foreground {
                loader.sprites.entry(*fg).or_insert_with(|| {
                    Self::sprite_info(
                        &atlas_wrappers,
                        fg,
                        if tile_info.names[0].0.starts_with("t_tree")
                            || tile_info.names[0].0.starts_with("t_fence")
                            || tile_info.names[0].0.starts_with("t_splitrail_fence")
                            || tile_info.names[0].0.starts_with("t_shrub")
                            || tile_info.names[0].0.starts_with("t_wall")
                            || tile_info.names[0].0.starts_with("t_door")
                            || tile_info.names[0].0.starts_with("t_flower")
                        {
                            SpriteOrientation::Vertical
                        } else {
                            SpriteOrientation::Horizontal
                        },
                        SpriteLayer::Front,
                    )
                });
            }
            for bg in &tile_info.background {
                loader.sprites.entry(*bg).or_insert_with(|| {
                    Self::sprite_info(
                        &atlas_wrappers,
                        bg,
                        if tile_info.names[0].0.starts_with("not_t_tree") {
                            SpriteOrientation::Vertical
                        } else {
                            SpriteOrientation::Horizontal
                        },
                        SpriteLayer::Back,
                    )
                });
            }
        }

        loader
    }

    fn sprite_info(
        atlas_wrappers: &[AtlasWrapper],
        sprite_number: &SpriteNumber,
        orientation: SpriteOrientation,
        layer: SpriteLayer,
    ) -> SpriteInfo {
        atlas_wrappers
            .iter()
            .find(|atlas_wrapper| atlas_wrapper.contains(sprite_number))
            .map(|atlas_wrapper| SpriteInfo {
                mesh_info: MeshInfo {
                    index: (*sprite_number).to_usize() - atlas_wrapper.range.0.to_usize(),
                    width: match &atlas_wrapper.imagepath {
                        p if p.ends_with("filler_tall.png") => 2,
                        p if p.ends_with("large_ridden.png") => 3,
                        p if p.ends_with("giant.png") => 4,
                        p if p.ends_with("huge.png") => 4,
                        p if p.ends_with("large.png") => 8,
                        p if p.ends_with("centered.png") => 12,
                        p if p.ends_with("small.png") => 12,
                        _ => 16,
                    },
                    size: 1 + atlas_wrapper.range.1.to_usize() - atlas_wrapper.range.0.to_usize(),
                },
                imagepath: atlas_wrapper.imagepath.clone(),
                orientation,
                layer,
                scale: atlas_wrapper.scale,
                offset: atlas_wrapper.offset,
            })
            /*.map(|sprite_info| {
                dbg!(&sprite_info);
                sprite_info
            })*/
            .unwrap_or_else(|| panic!("{sprite_number:?} not found"))
    }

    pub fn sprite_infos(&self, tile_name: &TileName) -> Vec<SpriteInfo> {
        let mut bundles = Vec::new();
        let (foreground, background) = tile_name
            .variants()
            .iter()
            .find_map(|variant| self.tiles.get(variant))
            .unwrap_or_else(|| self.tiles.get(&TileName::new("unknown")).unwrap())
            .sprite_numbers();
        if tile_name.0.as_str() != "t_dirt" && !tile_name.0.starts_with("t_grass") {
            println!("{tile_name:?} {foreground:?} {background:?}");
        }
        bundles.extend(foreground.map(|fg| self.sprites[&fg].clone()));
        bundles.extend(background.map(|bg| self.sprites[&bg].clone()));
        bundles
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
