use crate::tile::atlas::CddaAtlas;
use serde::Deserialize;
use serde_json::Value as JsonValue;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CddaTileConfig {
    #[expect(unused)]
    tile_info: JsonValue,

    #[serde(rename = "tiles-new")]
    pub atlases: Vec<CddaAtlas>,
}
