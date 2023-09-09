use crate::prelude::*;
use bevy::{
    asset::{AssetLoader, BoxedFuture, Error, LoadContext, LoadedAsset},
    reflect::{TypePath, TypeUuid},
};
use serde::Deserialize;
use std::{str::from_utf8, sync::OnceLock};

pub(crate) type OvermapBufferPath = PathFor<OvermapBuffer>;

impl OvermapBufferPath {
    pub(crate) fn new(sav_path: &SavPath, overzone: Overzone) -> Self {
        let mut seen_path = sav_path.0.clone();
        seen_path.set_extension(format!("seen.{}.{}", overzone.x, overzone.z));
        Self::init(seen_path)
    }
}

/** Corresponds to an 'overmapbuffer' in CDDA. It defines the save-specific information of a `Zone`. */
#[derive(Debug, Deserialize, TypePath, TypeUuid)]
#[serde(deny_unknown_fields)]
#[type_path = "cdda::world::OvermapBuffer"]
#[uuid = "e1ddd167-19aa-4abd-bdb6-bddcb65bc3ca"]
pub(crate) struct OvermapBuffer {
    /// Visible on the overmap
    pub(crate) visible: [RepetitionBlock<bool>; Level::AMOUNT],

    /// Marked as 'Exmplored' on the overmap
    #[allow(unused)]
    pub(crate) explored: [RepetitionBlock<bool>; Level::AMOUNT],

    #[allow(unused)]
    pub(crate) notes: Vec<serde_json::Value>,

    #[allow(unused)]
    pub(crate) extras: Vec<serde_json::Value>,
}

#[derive(Default)]
pub(crate) struct OvermapBufferLoader;

impl OvermapBufferLoader {
    const EXTENSION_MAX: usize = 1000;
    const EXTENSION_COUNT: usize = 2 * Self::EXTENSION_MAX + 1;
}

impl AssetLoader for OvermapBufferLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            let newline_pos = bytes
                .windows(1)
                .position(|window| window == b"\n")
                .expect("Version line");
            let after_first_line = bytes.split_at(newline_pos).1;
            let overmap_buffer_result = serde_json::from_slice::<OvermapBuffer>(after_first_line);
            let overmap_buffer = overmap_buffer_result.map_err(|e| {
                eprintln!(
                    "Overmap buffer loading error: {:?} {e:?}",
                    from_utf8(&bytes[0..40])
                );
                e
            })?;
            load_context.set_default_asset(LoadedAsset::new(overmap_buffer));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static STRINGS: OnceLock<[String; OvermapBufferLoader::EXTENSION_COUNT]> = OnceLock::new();
        static EXTENSIONS: OnceLock<[&str; OvermapBufferLoader::EXTENSION_COUNT]> = OnceLock::new();

        EXTENSIONS.get_or_init(|| {
            let strings = STRINGS.get_or_init(|| {
                let mut i = -(Self::EXTENSION_MAX as isize);
                [(); Self::EXTENSION_COUNT].map(|_| {
                    let string = i.to_string();
                    i += 1;
                    string
                })
            });

            let mut j = 0;
            [(); Self::EXTENSION_COUNT].map(|_| {
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
        let extensions = OvermapBufferLoader.extensions();
        assert_eq!(
            extensions.len(),
            OvermapBufferLoader::EXTENSION_COUNT,
            "{extensions:?}"
        );
        assert_eq!(
            extensions[0],
            (-(OvermapBufferLoader::EXTENSION_MAX as isize))
                .to_string()
                .as_str(),
            "{extensions:?}"
        );
        assert_eq!(
            extensions.last().expect("many items"),
            &OvermapBufferLoader::EXTENSION_MAX.to_string().as_str(),
            "{extensions:?}"
        );
    }
}
