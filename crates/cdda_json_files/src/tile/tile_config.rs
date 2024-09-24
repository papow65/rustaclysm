use crate::tile::atlas::CddaAtlas;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CddaTileConfig {
    #[expect(unused)]
    tile_info: serde_json::Value,

    #[serde(rename = "tiles-new")]
    pub atlases: Vec<CddaAtlas>,
}
