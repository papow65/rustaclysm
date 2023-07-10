use crate::prelude::*;
use bevy::{
    asset::{AssetLoader, BoxedFuture, Error, LoadContext, LoadedAsset},
    reflect::{TypePath, TypeUuid},
};
use serde::Deserialize;

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
                    std::str::from_utf8(&bytes[0..40])
                );
                e
            })?;
            load_context.set_default_asset(LoadedAsset::new(overmap_buffer));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        // TODO not manually
        // TODO larger range
        // TODO make this irrelevant
        &[
            "-30", "-29", "-28", "-27", "-26", "-25", "-24", "-23", "-22", "-21", "-20", "-19",
            "-18", "-17", "-16", "-15", "-14", "-13", "-12", "-11", "-10", "-9", "-8", "-7", "-6",
            "-5", "-4", "-3", "-2", "-1", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10",
            "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23", "24",
            "25", "26", "27", "28", "29", "30",
        ]
    }
}
