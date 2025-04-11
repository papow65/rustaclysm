use crate::{Alternative, ComponentPresence, Ignored, InfoId, RequiredQualities, ToolPresence};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Requirement {
    pub id: InfoId<Requirement>,

    #[serde(default)]
    pub qualities: RequiredQualities,

    #[serde(default)]
    pub components: Vec<Vec<Alternative<ComponentPresence>>>,

    #[serde(default)]
    pub tools: Vec<Vec<Alternative<ToolPresence>>>,

    #[serde(flatten)]
    _ignored: Ignored<Self>,
}
