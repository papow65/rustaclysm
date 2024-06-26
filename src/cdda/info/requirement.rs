use crate::prelude::{Alternative, RequiredQualities};
use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Requirement {
    #[serde(default)]
    pub(crate) qualities: RequiredQualities,

    #[serde(default)]
    pub(crate) components: Vec<Vec<Alternative>>,

    #[allow(unused)]
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}
