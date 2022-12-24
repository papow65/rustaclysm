use crate::prelude::{FlatVec, ObjectId};
use serde::Deserialize;

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub(crate) struct Field {
    pub(crate) id: ObjectId,
    intensity: i32,
    age: u64,
}

pub(crate) type FieldVec = FlatVec<Field, 3>;
