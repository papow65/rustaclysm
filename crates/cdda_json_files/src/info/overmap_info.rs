use crate::{HashMap, InfoId, ItemName};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct OvermapInfo {
    pub name: ItemName,
    pub looks_like: Option<InfoId>,

    #[expect(unused)]
    #[serde(flatten)]
    extra: HashMap<Arc<str>, serde_json::Value>,
}
