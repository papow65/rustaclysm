use crate::structure::FlatVec;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct At<T> {
    pub x: u8,
    pub y: u8,
    pub obj: T,
}

pub type AtVec<T> = FlatVec<At<T>, 3>;
