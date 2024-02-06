use bevy::asset::{io::Reader, Asset, AssetLoader, BoxedFuture, LoadContext};
use either::Either;
use futures_lite::AsyncReadExt;
use serde::Deserialize;
use std::{marker::PhantomData, str::from_utf8, sync::OnceLock};

/** This loads both overmaps and overmap buffers, since those have the same extensions. */
//#[derive(Default)]
pub(crate) struct OvermapLoader<T>(PhantomData<T>)
where
    T: Asset;

impl<T> AssetLoader for OvermapLoader<T>
where
    for<'de> T: Asset + Deserialize<'de>,
{
    type Asset = T;
    type Settings = ();
    type Error = Either<std::io::Error, serde_json::Error>;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader
                .read_to_end(&mut bytes)
                .await
                .map_err(|e| {
                    eprintln!("Map file loading error: {:?} {e:?}", load_context.path(),);
                    e
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
                .map_err(|e| {
                    eprintln!(
                        "Overmap (buffer?) loading error: {file_name:?} {:?} {e:?}",
                        from_utf8(&bytes[0..40])
                    );
                    e
                })
                .map_err(Either::Right)
        })
    }

    fn extensions(&self) -> &[&str] {
        extensions()
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

const EXTENSION_MAX: usize = 1000;
const EXTENSION_COUNT: usize = 2 * EXTENSION_MAX + 1;

fn extensions() -> &'static [&'static str] {
    static STRINGS: OnceLock<[String; EXTENSION_COUNT]> = OnceLock::new();
    static EXTENSIONS: OnceLock<[&str; EXTENSION_COUNT]> = OnceLock::new();

    EXTENSIONS.get_or_init(|| {
        let strings = STRINGS.get_or_init(|| {
            let mut i = -(EXTENSION_MAX as isize);
            [(); EXTENSION_COUNT].map(|()| {
                let string = i.to_string();
                i += 1;
                string
            })
        });

        let mut j = 0;
        [(); EXTENSION_COUNT].map(|()| {
            let extension = strings[j].as_str();
            j += 1;
            extension
        })
    })
}

#[cfg(test)]
mod overmap_buffer_tests {
    use super::*;
    #[test]
    fn check_extensions() {
        let extensions = extensions();
        assert_eq!(extensions.len(), EXTENSION_COUNT, "{extensions:?}");
        assert_eq!(
            extensions[0],
            (-(EXTENSION_MAX as isize)).to_string().as_str(),
            "{extensions:?}"
        );
        assert_eq!(
            extensions.last().expect("many items"),
            &EXTENSION_MAX.to_string().as_str(),
            "{extensions:?}"
        );
    }
}
