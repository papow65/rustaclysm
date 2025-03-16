use crate::{HashMap, ItemName, UntypedInfoId};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct OvermapTerrainInfo {
    pub name: ItemName,
    pub looks_like: Option<UntypedInfoId>,

    #[expect(unused)]
    #[serde(flatten)]
    extra: HashMap<Arc<str>, serde_json::Value>,
}
