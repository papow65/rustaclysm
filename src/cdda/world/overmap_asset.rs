use crate::prelude::*;
use bevy::{
    asset::{io::Reader, Asset, AssetLoader, BoxedFuture, LoadContext},
    reflect::TypePath,
};
use either::Either;
use futures_lite::AsyncReadExt;
use std::{fmt, str::from_utf8, sync::OnceLock};

#[derive(Debug, Asset, TypePath)]
pub(crate) struct OvermapAsset(OvermapEnum);

/*
-#[derive(Debug, Deserialize, TypePath, TypeUuid)]
+#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
-#[type_path = "cdda::world::Overmap"]
-#[uuid = "a4067c84-4c64-4765-9000-53a045919796"]
*/

impl OvermapAsset {
    pub(crate) fn overmap(&self, panic_hint: impl fmt::Debug) -> &Overmap {
        let OvermapEnum::Overmap(ref overmap) = self.0 else {
            panic!("{panic_hint:?} {self:?}");
        };
        overmap
    }

    pub(crate) fn buffer(&self, panic_hint: impl fmt::Debug) -> &OvermapBuffer {
        let OvermapEnum::Buffer(ref buffer) = self.0 else {
            panic!("{panic_hint:?} {self:?}");
        };
        buffer
    }
}

#[derive(Debug)]
pub(crate) enum OvermapEnum {
    Overmap(Overmap),
    Buffer(OvermapBuffer),
}

/** This loads both overmaps and overmap buffers, since those have the same extensions. */
#[derive(Default)]
pub(crate) struct OvermapLoader;

impl OvermapLoader {
    const EXTENSION_MAX: usize = 1000;
    const EXTENSION_COUNT: usize = 2 * Self::EXTENSION_MAX + 1;
}

impl AssetLoader for OvermapLoader {
    type Asset = OvermapAsset;
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

            Ok(OvermapAsset(
                if file_name.starts_with("o.") {
                    serde_json::from_slice::<Overmap>(after_first_line).map(OvermapEnum::Overmap)
                } else {
                    serde_json::from_slice::<OvermapBuffer>(after_first_line)
                        .map(OvermapEnum::Buffer)
                }
                .map_err(|e| {
                    eprintln!(
                        "Overmap (buffer?) loading error: {file_name:?} {:?} {e:?}",
                        from_utf8(&bytes[0..40])
                    );
                    e
                })
                .map_err(Either::Right)?,
            ))
        })
    }

    fn extensions(&self) -> &[&str] {
        static STRINGS: OnceLock<[String; OvermapLoader::EXTENSION_COUNT]> = OnceLock::new();
        static EXTENSIONS: OnceLock<[&str; OvermapLoader::EXTENSION_COUNT]> = OnceLock::new();

        EXTENSIONS.get_or_init(|| {
            let strings = STRINGS.get_or_init(|| {
                let mut i = -(Self::EXTENSION_MAX as isize);
                [(); Self::EXTENSION_COUNT].map(|()| {
                    let string = i.to_string();
                    i += 1;
                    string
                })
            });

            let mut j = 0;
            [(); Self::EXTENSION_COUNT].map(|()| {
                let extension = strings[j].as_str();
                j += 1;
                extension
            })
        })
    }
}

#[cfg(test)]
mod overmap_buffer_tests {
    use super::*;
    #[test]
    fn check_extensions() {
        let extensions = OvermapLoader.extensions();
        assert_eq!(
            extensions.len(),
            OvermapLoader::EXTENSION_COUNT,
            "{extensions:?}"
        );
        assert_eq!(
            extensions[0],
            (-(OvermapLoader::EXTENSION_MAX as isize))
                .to_string()
                .as_str(),
            "{extensions:?}"
        );
        assert_eq!(
            extensions.last().expect("many items"),
            &OvermapLoader::EXTENSION_MAX.to_string().as_str(),
            "{extensions:?}"
        );
    }
}
