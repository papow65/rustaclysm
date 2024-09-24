use crate::{Overmap, RepetitionBlock};
use bevy::{asset::Asset, reflect::TypePath};
use serde::Deserialize;

/// Corresponds to an 'overmapbuffer' in CDDA. It defines the save-specific information of an `OverZone`.
#[derive(Debug, Deserialize, Asset, TypePath)]
#[serde(deny_unknown_fields)]
pub struct OvermapBuffer {
    /// Visible on the overmap
    pub visible: [RepetitionBlock<bool>; Overmap::LEVEL_AMOUNT],

    /// Marked as 'Exmplored' on the overmap
    pub explored: [RepetitionBlock<bool>; Overmap::LEVEL_AMOUNT],

    pub notes: Vec<serde_json::Value>,
    pub extras: Vec<serde_json::Value>,
}
