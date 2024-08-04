use crate::prelude::{Level, Overzone, PathFor, RepetitionBlock, SavPath};
use bevy::{asset::Asset, reflect::TypePath};
use serde::Deserialize;

pub(crate) type OvermapBufferPath = PathFor<OvermapBuffer>;

impl OvermapBufferPath {
    pub(crate) fn new(sav_path: &SavPath, overzone: Overzone) -> Self {
        let mut seen_path = sav_path.0.clone();
        seen_path.set_extension(format!("seen.{}.{}", overzone.x, overzone.z));
        Self::init(seen_path)
    }
}

/// Corresponds to an 'overmapbuffer' in CDDA. It defines the save-specific information of an `OverZone`.
#[derive(Debug, Deserialize, Asset, TypePath)]
#[serde(deny_unknown_fields)]
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
