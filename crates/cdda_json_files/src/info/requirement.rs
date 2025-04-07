use crate::{Alternative, Ignored, InfoId, RequiredQualities};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Requirement {
    pub id: InfoId<Requirement>,

    #[serde(default)]
    pub qualities: RequiredQualities,

    #[serde(default)]
    pub components: Vec<Vec<Alternative<u32>>>,

    #[serde(default)]
    pub tools: Vec<Vec<Alternative<i32>>>,

    #[serde(flatten)]
    _ignored: Ignored<Self>,
}
