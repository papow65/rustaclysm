use crate::{ItemName, ObjectId};
use bevy::utils::HashMap;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub struct FieldInfo {
    pub intensity_levels: Vec<IntensityLevel>,
    pub looks_like: Option<ObjectId>,

    #[expect(unused)]
    #[serde(flatten)]
    extra: HashMap<Arc<str>, serde_json::Value>,
}

impl FieldInfo {
    pub fn name(&self) -> &ItemName {
        self.intensity_levels[0]
            .name
            .as_ref()
            .expect("Named first level")
    }
}

#[derive(Debug, Deserialize)]
pub struct IntensityLevel {
    name: Option<ItemName>,

    #[expect(unused)]
    #[serde(flatten)]
    extra: HashMap<Arc<str>, serde_json::Value>,
}
