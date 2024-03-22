use crate::prelude::ObjectId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Migration {
    #[serde(rename(deserialize = "type"))]
    #[allow(dead_code)]
    type_: String,

    pub(crate) replace: ObjectId,
}
