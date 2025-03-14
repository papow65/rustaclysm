use crate::{FieldInfo, FlatVec, RequiredLinkedLater};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Field {
    #[serde(rename = "id")]
    pub field_info: RequiredLinkedLater<FieldInfo>,

    #[expect(unused)]
    intensity: i32,
    #[expect(unused)]
    age: i64,
}

pub type FieldVec = FlatVec<Field, 3>;
