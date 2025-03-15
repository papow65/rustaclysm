use crate::{HashMap, InfoId, ItemName, UntypedInfoId};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub struct FieldInfo {
    pub id: InfoId<Self>,
    pub intensity_levels: Vec<IntensityLevel>,
    pub looks_like: Option<UntypedInfoId>,

    #[expect(unused)]
    #[serde(flatten)]
    extra: HashMap<Arc<str>, serde_json::Value>,
}

impl FieldInfo {
    #[must_use]
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
