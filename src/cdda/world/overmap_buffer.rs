use crate::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
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
