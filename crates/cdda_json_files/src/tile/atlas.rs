use crate::tile::tile_info::TileInfo;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CddaAtlas {
    pub file: Arc<str>,
    pub sprite_width: Option<u8>,
    pub sprite_height: Option<u8>,
    pub sprite_offset_x: Option<i8>,
    pub sprite_offset_y: Option<i8>,
    pub tiles: Vec<TileInfo>,

    pub ascii: Option<serde_json::Value>,

    #[serde(rename = "//")]
    pub comment: Option<Arc<str>>,
}
