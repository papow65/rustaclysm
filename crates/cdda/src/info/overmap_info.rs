use crate::ItemName;
use crate::ObjectId;
use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OvermapInfo {
    pub name: ItemName,
    pub looks_like: Option<ObjectId>,

    #[expect(unused)]
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}
