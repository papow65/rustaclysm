use crate::gameplay::MapAsset;
use crate::gameplay::cdda::error::Error;
use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use std::{str::from_utf8, sync::Arc};

#[derive(Default)]
pub(super) struct MapLoader;

impl AssetLoader for MapLoader {
    type Asset = MapAsset;
    type Settings = ();
    type Error = Error;

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
            .map_err(|err| Error::Io { _wrapped: err })?;

        let map =
            serde_json::from_slice::<MapAsset>(&bytes).map_err(|err| Error::JsonWithContext {
                _wrapped: err,
                _file_path: load_context.path().to_path_buf(),
                _contents: Arc::from(from_utf8(&bytes[0..1000]).unwrap_or("(invalid UTF8)")),
            })?;
        Ok(map)
    }

    fn extensions(&self) -> &[&str] {
        &["map"]
    }
}
