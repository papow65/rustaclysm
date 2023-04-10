use crate::prelude::{ItemName, ObjectId};
use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub(crate) struct FieldInfo {
    pub(crate) intensity_levels: Vec<IntensityLevel>,
    pub(crate) looks_like: Option<ObjectId>,

    #[allow(unused)]
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

impl FieldInfo {
    pub(crate) fn name(&self) -> &ItemName {
        self.intensity_levels[0].name.as_ref().unwrap()
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct IntensityLevel {
    name: Option<ItemName>,

    #[allow(unused)]
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}
