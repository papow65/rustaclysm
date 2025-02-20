use bevy::asset::{Asset, AssetLoader, LoadContext, io::Reader};
use either::Either;
use serde::Deserialize;
use std::{marker::PhantomData, str::from_utf8};

pub(super) struct OvermapLoader<T>(PhantomData<T>)
where
    T: Asset;

impl<T> AssetLoader for OvermapLoader<T>
where
    for<'de> T: Asset + Deserialize<'de>,
{
    type Asset = T;
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
                eprintln!("Map file loading error: {:?} {e:?}", load_context.path());
            })
            .map_err(Either::Left)?;

        let newline_pos = bytes
            .windows(1)
            .position(|window| window == b"\n")
            .expect("Version line");
        let after_first_line = bytes.split_at(newline_pos).1;

        let file_name = load_context
            .path()
            .file_name()
            .expect("File name present")
            .to_str()
            .expect("Unicode filename");

        serde_json::from_slice::<T>(after_first_line)
            .inspect_err(|e| {
                eprintln!(
                    "Overmap (buffer?) loading error: {file_name:?} {:?} {e:?}",
                    from_utf8(&bytes[0..40])
                );
            })
            .map_err(Either::Right)
    }

    fn extensions(&self) -> &[&str] {
        &[]
    }
}

// `#[derive(Default)]` does not work.
impl<'de, T> Default for OvermapLoader<T>
where
    T: Asset + Deserialize<'de>,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}
