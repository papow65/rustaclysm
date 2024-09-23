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

#[cfg(test)]
mod recipe_tests {
    use super::*;
    #[test]
    #[ignore] // Not a proper unit test, because the config tile may not exist
    fn it_works() {
        let json = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/gfx/UltimateCataclysm/tile_config.json"
        ));
        let result = serde_json::from_str::<CddaTileConfig>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
