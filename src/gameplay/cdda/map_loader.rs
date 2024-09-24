use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use cdda_json_files::{Error, Map};
use futures_lite::AsyncReadExt;
use std::{str::from_utf8, sync::Arc};

#[derive(Default)]
pub(crate) struct MapLoader;

impl AssetLoader for MapLoader {
    type Asset = Map;
    type Settings = ();
    type Error = Error;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|err| Error::Io { _wrapped: err })?;

        let map = serde_json::from_slice::<Map>(&bytes).map_err(|err| Error::Json {
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
