use crate::{HashMap, ItemName, ObjectId};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct OvermapInfo {
    pub name: ItemName,
    pub looks_like: Option<ObjectId>,

    #[expect(unused)]
    #[serde(flatten)]
    extra: HashMap<Arc<str>, serde_json::Value>,
}
