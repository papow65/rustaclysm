use crate::ObjectId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Migration {
    #[serde(rename(deserialize = "type"))]
    #[expect(dead_code)]
    type_: String,

    pub replace: ObjectId,
}
