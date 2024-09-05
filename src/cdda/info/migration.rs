use crate::gameplay::ObjectId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Migration {
    #[serde(rename(deserialize = "type"))]
    #[expect(dead_code)]
    type_: String,

    pub(crate) replace: ObjectId,
}
