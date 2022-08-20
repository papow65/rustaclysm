use crate::prelude::{FlatVec, ObjectName};
use serde::Deserialize;

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub(crate) struct Field {
    pub(crate) tile_name: ObjectName,
    intensity: i32,
    age: u64,
}

pub(crate) type FieldVec = FlatVec<Field, 3>;
