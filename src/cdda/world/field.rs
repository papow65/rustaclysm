use crate::{cdda::FlatVec, gameplay::ObjectId};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Field {
    pub(crate) id: ObjectId,

    #[allow(unused)]
    intensity: i32,
    #[allow(unused)]
    age: i64,
}

pub(crate) type FieldVec = FlatVec<Field, 3>;
