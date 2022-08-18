use crate::prelude::*;
use serde::Deserialize;
use std::fs::read_to_string;

/** Corresponds to an 'overmapbuffer' in CDDA. It defines the save-specific information of a `Zone`. */
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OvermapBuffer {
    /// Visible on the overmap (?)
    pub visible: Vec<RepetitionBlock<bool>>,

    /// Marked as 'Exmplored' on the overmap
    pub explored: Vec<RepetitionBlock<bool>>,

    pub notes: Vec<serde_json::Value>,
    pub extras: Vec<serde_json::Value>,
}

impl TryFrom<Overzone> for OvermapBuffer {
    type Error = ();
    fn try_from(overzone: Overzone) -> Result<Self, ()> {
        let filepath = format!("assets/save/#VGFsZG9y.seen.{}.{}", overzone.x, overzone.z,);
        read_to_string(&filepath)
            .ok()
            .map(|s| {
                println!("Found overmap buffer: {filepath}");
                s
            })
            .map(|s| s.split_at(s.find('\n').unwrap()).1.to_string())
            .map(|s| serde_json::from_str::<Self>(s.as_str()).unwrap())
            .ok_or(())
    }
}
