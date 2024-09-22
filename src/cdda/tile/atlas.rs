use crate::cdda::tile::tile_info::{CddaTileInfo, TileInfo};
use crate::cdda::{tile::SpriteNumber, TextureInfo};
use crate::gameplay::{MeshInfo, ObjectId, Transform2d};
use bevy::{prelude::Vec2, utils::HashMap};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CddaAtlas {
    file: String,
    sprite_width: Option<u8>,
    sprite_height: Option<u8>,
    sprite_offset_x: Option<i8>,
    sprite_offset_y: Option<i8>,
    tiles: Vec<CddaTileInfo>,

    #[expect(unused)]
    ascii: Option<serde_json::Value>,

    #[serde(rename = "//")]
    comment: Option<String>,
}

#[derive(Debug)]
pub(super) struct Atlas {
    range: (SpriteNumber, SpriteNumber),
    image_path: PathBuf,
    transform2d: Transform2d,
}

impl Atlas {
    pub(super) fn new(
        tileset_path: &Path,
        cdda_atlas: CddaAtlas,
        tiles: &mut HashMap<ObjectId, TileInfo>,
    ) -> Self {
        let filename = cdda_atlas.file.as_str();
        let image_path = tileset_path.join(filename);

        let from_to = if let Some(comment) = cdda_atlas.comment {
            comment
                .split(' ')
                .flat_map(str::parse)
                .map(SpriteNumber)
                .collect::<Vec<SpriteNumber>>()
        } else {
            vec![SpriteNumber(0), SpriteNumber(1024)]
        };

        let width = cdda_atlas.sprite_width.map_or(1.0, |w| f32::from(w) / 32.0);
        let height = cdda_atlas
            .sprite_height
            .map_or(1.0, |h| f32::from(h) / 32.0);

        let offset_x = cdda_atlas
            .sprite_offset_x
            .map_or(0.0, |x| f32::from(x) / 32.0)
            + (0.5 * width - 0.5);

        // notice the minus sign
        let offset_y = -(cdda_atlas
            .sprite_offset_y
            .map_or(0.0, |y| f32::from(y) / 32.0)
            + (0.5 * height - 0.5));

        for tile in cdda_atlas.tiles {
            let tile_info = TileInfo::from(tile);
            for id in tile_info.ids() {
                tiles.insert(id.clone(), tile_info.clone());
            }
        }

        Self {
            range: (from_to[0], from_to[1]),
            image_path,
            transform2d: Transform2d {
                scale: Vec2::new(width, height),
                offset: Vec2::new(offset_x, offset_y),
            },
        }
    }

    pub(super) fn contains(&self, sprite_number: SpriteNumber) -> bool {
        (self.range.0..=self.range.1).contains(&sprite_number)
    }

    pub(super) fn texture_info(&self, sprite_number: SpriteNumber) -> TextureInfo {
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
