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
    pub(crate) visible: [RepetitionBlock<bool>; Level::AMOUNT],

    /// Marked as 'Exmplored' on the overmap
    #[allow(unused)]
    pub(crate) explored: [RepetitionBlock<bool>; Level::AMOUNT],

    #[allow(unused)]
    pub(crate) notes: Vec<serde_json::Value>,

    #[allow(unused)]
    pub(crate) extras: Vec<serde_json::Value>,
}

impl OvermapBuffer {
    pub(crate) fn fallback() -> Self {
        Self {
            visible: [(); Level::AMOUNT].map(|_| {
                RepetitionBlock::new(CddaAmount {
                    obj: false,
                    amount: 180 * 180,
                })
            }),
            explored: [(); Level::AMOUNT].map(|_| {
                RepetitionBlock::new(CddaAmount {
                    obj: false,
                    amount: 180 * 180,
                })
            }),
            notes: Vec::new(),
            extras: Vec::new(),
        }
    }
}

impl TryFrom<OvermapBufferPath> for OvermapBuffer {
    type Error = ();
    fn try_from(overmap_buffer_path: OvermapBufferPath) -> Result<Self, ()> {
        //println!("Path: {overmap_buffer_path}");
        read_to_string(overmap_buffer_path.0)
            .ok()
            .map(|s| {
                let first_newline = s.find('\n').unwrap();
                let after_first_line = s.split_at(first_newline).1;
                serde_json::from_str(after_first_line).unwrap_or_else(|err| panic!("{err:?}"))
            })
            .ok_or(())
    }
}
