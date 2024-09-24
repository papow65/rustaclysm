use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use cdda_json_files::MapMemory;
use either::Either;
use futures_lite::AsyncReadExt;
use std::str::from_utf8;

#[derive(Default)]
pub(crate) struct MapMemoryLoader;

impl AssetLoader for MapMemoryLoader {
    type Asset = MapMemory;
    type Settings = ();
    type Error = Either<std::io::Error, serde_json::Error>;

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
            .inspect_err(|e| {
                eprintln!("Map file loading error: {:?} {e:?}", load_context.path(),);
            })
            .map_err(Either::Left)?;

        let map_memory = serde_json::from_slice::<MapMemory>(&bytes)
            .map_err(|e| {
                eprintln!(
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
