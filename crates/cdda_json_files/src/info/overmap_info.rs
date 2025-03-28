use crate::{ItemName, UntypedInfoId};
use bevy_platform_support::collections::HashMap;
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
