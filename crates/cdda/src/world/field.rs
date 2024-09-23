use crate::{FlatVec, ObjectId};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Field {
    pub id: ObjectId,

    #[expect(unused)]
    intensity: i32,
    #[expect(unused)]
    age: i64,
}

pub type FieldVec = FlatVec<Field, 3>;
