use crate::prelude::{ItemName, ObjectId};
use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct OvermapInfo {
    pub(crate) name: ItemName,
    pub(crate) looks_like: Option<ObjectId>,

    #[allow(unused)]
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}
