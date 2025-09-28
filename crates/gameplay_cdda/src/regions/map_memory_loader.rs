use crate::MapMemoryAsset;
use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use bevy::prelude::error;
use either::Either;
use serde_json::from_slice as from_json_slice;
use std::str::from_utf8;

#[derive(Default)]
pub(super) struct MapMemoryLoader;

impl AssetLoader for MapMemoryLoader {
    type Asset = MapMemoryAsset;
    type Settings = ();
    type Error = Either<std::io::Error, serde_json::Error>;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .inspect_err(|e| {
                error!("Map file loading error: {:?} {e:?}", load_context.path());
            })
            .map_err(Either::Left)?;

        let map_memory = from_json_slice::<MapMemoryAsset>(&bytes)
            .map_err(|e| {
                error!(
                    "Map memory json loading error: {:?} {:?} {e:?}",
                    load_context.path(),
                    from_utf8(&bytes[0..40])
                );
                e
            })
            .map_err(Either::Right)?;
        Ok(map_memory)
    }

    fn extensions(&self) -> &[&str] {
        &["mmr"]
    }
}
