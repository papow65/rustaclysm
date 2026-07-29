use crate::MapAsset;
use anyhow::Error as AnyError;
use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use bevy::prelude::TypePath;
use serde_json::from_slice as from_json_slice;
use std::str::from_utf8;
use util::json_parse_error;

#[derive(Default, TypePath)]
pub(super) struct MapLoader;

impl AssetLoader for MapLoader {
    type Asset = MapAsset;
    type Settings = ();
    type Error = AnyError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        from_json_slice::<MapAsset>(&bytes).map_err(|err| {
            json_parse_error(
                err,
                load_context.path().path(),
                from_utf8(&bytes).unwrap_or("(invalid UTF8)"),
            )
        })
    }

    fn extensions(&self) -> &[&str] {
        &["map"]
    }
}
