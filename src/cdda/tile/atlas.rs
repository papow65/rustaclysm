use crate::cdda::{tile::tile_info::TileInfo, tile::SpriteNumber, Error, TextureInfo};
use crate::common::Paths;
use crate::gameplay::{MeshInfo, ObjectId, Transform2d};
use bevy::{prelude::Vec2, utils::HashMap};
use std::path::PathBuf;

#[derive(Debug)]
pub(super) struct Atlas {
    range: (SpriteNumber, SpriteNumber),
    image_path: PathBuf,
    transform2d: Transform2d,
}

impl Atlas {
    pub(super) fn try_new(
        json: &serde_json::Value,
        tiles: &mut HashMap<ObjectId, TileInfo>,
    ) -> Result<Self, Error> {
        let atlas = json
            .as_object()
            .expect("JSON value should be an object (map)");
        let filename = atlas["file"]
            .as_str()
            .expect("'file' key should be present");
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
            for name in tile_info.names() {
                tiles.insert(name.clone(), tile_info.clone());
            }
        }

        Ok(Self {
            range: (from_to[0], from_to[1]),
            image_path,
            transform2d: Transform2d {
                scale: Vec2::new(width, height),
                offset: Vec2::new(offset_x, offset_y),
            },
        })
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
