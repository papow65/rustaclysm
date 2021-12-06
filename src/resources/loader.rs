use std::fs::read_to_string;

use rand::seq::SliceRandom;

use serde::de::{Deserializer, SeqAccess, Visitor};
use serde::Deserialize;

use bevy::prelude::*;
use bevy::utils::HashMap;

use super::super::components::*;

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct TileName(String);

impl TileName {
    pub fn new(value: &'static str) -> Self {
        Self(value.into())
    }

    pub fn to_label(&self) -> Label {
        Label::new(self.0.clone())
    }

    pub fn variants(&self) -> Vec<Self> {
        vec![
            Self(self.0.clone() + "_season_summer"),
            self.clone(),
            Self(self.0[..self.0.rfind('_').unwrap()].to_string()),
        ]
    }
}

#[derive(Debug)]
pub struct TileInfo {
    pub names: Vec<TileName>,
    pub foreground: Vec<SpriteNumber>,
    pub background: Vec<SpriteNumber>,
}

impl TileInfo {
    fn sprite_number(&self) -> SpriteNumber {
        self.foreground
            .choose(&mut rand::thread_rng())
            .or_else(|| self.background.choose(&mut rand::thread_rng()))
            .copied()
            .unwrap()
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

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub struct SpriteNumber(u32);

impl SpriteNumber {
    fn from_json(value: &serde_json::Value) -> Self {
        Self(value.as_u64().unwrap() as u32)
    }

    fn from_number(n: &serde_json::Number) -> Self {
        Self(n.as_u64().unwrap() as u32)
    }

    pub fn to_sprite(self, atlas_wrapper: &AtlasWrapper) -> TextureAtlasSprite {
        assert!(
            atlas_wrapper.contains(&self),
            "{:?} {:?} {:?}",
            atlas_wrapper.filepath,
            atlas_wrapper.tile_infos,
            &self
        );
        TextureAtlasSprite::new(self.0 - atlas_wrapper.from.0)
    }
}

pub struct AtlasWrapper {
    pub filepath: String,
    pub handle: Handle<TextureAtlas>,
    pub from: SpriteNumber,
    pub to: SpriteNumber,
    // TODO
    // sprite_width: 20,
    // sprite_height: 20,
    // sprite_offset_x: 6,
    // sprite_offset_y: 0,
    pub tile_infos: HashMap<TileName, TileInfo>,
}

impl AtlasWrapper {
    fn contains(&self, sprite_number: &SpriteNumber) -> bool {
        (self.from..self.to).contains(sprite_number)
    }

    fn find_tile(&self, tile_name: &TileName) -> Option<TileInfo> {
        self.tile_infos.get(tile_name).cloned()
    }
}

pub struct Loader {
    pub atlas_wrappers: Vec<AtlasWrapper>,
}

impl Loader {
    pub fn new(texture_atlases: &mut Assets<TextureAtlas>, asset_server: &AssetServer) -> Self {
        let filepath = "assets/gfx/UltimateCataclysm/tile_config.json";
        let file_contents = read_to_string(&filepath).unwrap();
        let json: serde_json::Value = serde_json::from_str(&file_contents).unwrap();
        let atlases = json.as_object().unwrap()["tiles-new"].as_array().unwrap();

        let mut result = Self {
            atlas_wrappers: Vec::new(),
        };

        for atlas in atlases {
            let atlas = atlas.as_object().unwrap();

            let filename = atlas["file"].as_str().unwrap();
            if filename == "fallback.png" {
                continue;
            }
            let filepath = "gfx/UltimateCataclysm/".to_string() + filename;

            let width = atlas
                .get("sprite_width")
                .and_then(serde_json::Value::as_u64)
                .map(|w| w as f32 / 32.0)
                .unwrap_or(1.0);
            let height = atlas
                .get("sprite_height")
                .and_then(serde_json::Value::as_u64)
                .map(|h| h as f32 / 32.0)
                .unwrap_or(1.0);

            let handle = texture_atlases.add(TextureAtlas::from_grid(
                asset_server.load(filepath.as_str()),
                Vec2::new(width, height),
                16,
                225,
            ));

            let from_to = if let Some(comment) = atlas.get("//") {
                comment
                    .as_str()
                    .unwrap()
                    .split(' ')
                    .flat_map(str::parse)
                    .map(SpriteNumber)
                    .collect::<Vec<SpriteNumber>>()
            } else {
                vec![SpriteNumber(0), SpriteNumber(99999)]
            };

            let mut atlas_wrapper = AtlasWrapper {
                filepath,
                handle,
                from: from_to[0],
                to: from_to[1],
                tile_infos: HashMap::default(),
            };

            for tile in atlas["tiles"].as_array().unwrap() {
                let tile_info = load_tile_info(tile);
                for name in &tile_info.names {
                    atlas_wrapper
                        .tile_infos
                        .insert(name.clone(), tile_info.clone());
                }
            }

            result.atlas_wrappers.push(atlas_wrapper);
        }

        result
    }

    pub fn atlas_and_sprite(
        &self,
        tile_name: &TileName,
    ) -> (Handle<TextureAtlas>, TextureAtlasSprite) {
        for variant in &tile_name.variants() {
            for atlas_wrapper in &self.atlas_wrappers {
                if let Some(tile_info) = atlas_wrapper.find_tile(variant) {
                    let sprite_number = tile_info.sprite_number();
                    // tiles may be defined in another atlas
                    for atlas_wrapper in &self.atlas_wrappers {
                        if atlas_wrapper.contains(&sprite_number) {
                            return (
                                atlas_wrapper.handle.clone(),
                                sprite_number.to_sprite(atlas_wrapper),
                            );
                        }
                    }
                    panic!(
                        "Tile {:?} ({:?}, {:?}) not found",
                        tile_name, variant, sprite_number
                    );
                }
            }
        }
        panic!("Tile {:?} not found", tile_name);
    }

    pub fn zone_layout(zone_pos: Pos) -> Option<ZoneLayout> {
        let filepath = format!(
            "assets/maps/{}.{}.{}/{}.{}.{}.map",
            zone_pos.0 / 32,
            zone_pos.2 / 32,
            zone_pos.1,
            zone_pos.0,
            zone_pos.2,
            zone_pos.1
        );
        println!("Path: {}", filepath);
        read_to_string(&filepath)
            .ok()
            .map(|s| ZoneLayout::new(s.as_str()))
    }
}

#[derive(Debug)]
pub struct ZoneLayout {
    pub subzone_layouts: Vec<SubzoneLayout>,
}

#[derive(Debug, Deserialize)]
pub struct SubzoneLayout {
    pub version: u16,
    pub coordinates: (i16, i16, i16),
    pub turn_last_touched: u32,
    pub temperature: i16,

    #[serde(deserialize_with = "flatten_cdda_seq")]
    pub terrain: Vec<TileName>,

    pub furniture: Vec<serde_json::Value>,
    pub items: Vec<serde_json::Value>,
    pub traps: Vec<serde_json::Value>,
    pub fields: Vec<serde_json::Value>,
    pub cosmetics: Vec<serde_json::Value>,
    pub spawns: Vec<serde_json::Value>,
    pub vehicles: Vec<serde_json::Value>,
    pub partial_constructions: Vec<serde_json::Value>,
}

impl ZoneLayout {
    fn new(file_contents: &str) -> Self {
        Self {
            subzone_layouts: serde_json::from_str(file_contents).unwrap(),
        }
    }
}

fn flatten_cdda_seq<'de, D>(deserializer: D) -> Result<Vec<TileName>, D::Error>
where
    D: Deserializer<'de>,
{
    struct JsonStringsVisitor;

    impl<'de> Visitor<'de> for JsonStringsVisitor {
        type Value = Vec<TileName>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a (mixed) sequence of strings and lists with [string, amount]")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut result: Vec<TileName> = Vec::new();
            while let Some(element) = seq.next_element::<serde_json::Value>()? {
                match element {
                    serde_json::Value::String(s) => result.push(TileName(s.to_string())),
                    serde_json::Value::Array(list) => {
                        for _ in 0..list[1].as_u64().unwrap() {
                            result.push(TileName(list[0].as_str().unwrap().to_string()));
                        }
                    }
                    _ => panic!("{:?}", element),
                }
            }
            Ok(result)
        }
    }

    deserializer.deserialize_seq(JsonStringsVisitor)
}

fn load_tile_info(tile: &serde_json::Value) -> TileInfo {
    let tile = tile.as_object().unwrap();

    TileInfo {
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
                other => panic!("{:?}", other),
            };
            tile_names
        },
        foreground: load_xg(tile.get("fg")),
        background: load_xg(tile.get("bg")),
    }
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
                        other => panic!("{:?}", other),
                    }
                }
                ids
            }
            other => panic!("{:?}", other),
        }
    } else {
        Vec::new()
    }
}
