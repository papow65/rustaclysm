use crate::ObjectId;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct Migration {
    #[serde(rename(deserialize = "type"))]
    #[expect(dead_code)]
    type_: Arc<str>,

    pub replace: ObjectId,
}
