use crate::prelude::{FlatVec, Pos};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct At<T> {
    pub(crate) x: u8,
    pub(crate) y: u8,
    pub(crate) obj: T,
}

impl<T> At<T> {
    pub(crate) const fn get(&self, relative_pos: Pos) -> Option<&T> {
        if relative_pos.x as u8 == self.x && relative_pos.z as u8 == self.y {
            Some(&self.obj)
        } else {
            None
        }
    }
}

pub(crate) type AtVec<T> = FlatVec<At<T>, 3>;
