use crate::{cdda::FlatVec, gameplay::ObjectId};
use serde::Deserialize;

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub(crate) struct Field {
    pub(crate) id: ObjectId,
    intensity: i32,
    age: i64,
}

pub(crate) type FieldVec = FlatVec<Field, 3>;
