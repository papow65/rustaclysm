use crate::prelude::*;
use serde::Deserialize;
use std::fs::read_to_string;

pub(crate) type OvermapBufferPath = PathFor<OvermapBuffer>;

impl OvermapBufferPath {
    pub(crate) fn new(sav_path: &SavPath, overzone: Overzone) -> Self {
        let mut seen_path = sav_path.0.clone();
        seen_path.set_extension(format!("seen.{}.{}", overzone.x, overzone.z));
        Self::init(seen_path)
    }
}

/** Corresponds to an 'overmapbuffer' in CDDA. It defines the save-specific information of a `Zone`. */
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OvermapBuffer {
    /// Visible on the overmap
    pub(crate) visible: Vec<RepetitionBlock<bool>>,

    /// Marked as 'Exmplored' on the overmap
    #[allow(unused)]
    pub(crate) explored: Vec<RepetitionBlock<bool>>,

    #[allow(unused)]
    pub(crate) notes: Vec<serde_json::Value>,

    #[allow(unused)]
    pub(crate) extras: Vec<serde_json::Value>,
}

impl TryFrom<OvermapBufferPath> for OvermapBuffer {
    type Error = serde_json::Error;
    fn try_from(overmap_buffer_path: OvermapBufferPath) -> Result<Self, Self::Error> {
        let file_contents =
            read_to_string(&overmap_buffer_path.0).expect("Overmap buffer file not found");
        println!("Found overmap buffer: {}", overmap_buffer_path.0.display());
        let from_second_line = file_contents.split_at(file_contents.find('\n').unwrap()).1;
        serde_json::from_str::<Self>(from_second_line)
    }
}
