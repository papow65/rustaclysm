use crate::{Ignored, ItemName, UntypedInfoId};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OvermapTerrainInfo {
    pub name: ItemName,
    pub looks_like: Option<UntypedInfoId>,

    #[serde(flatten)]
    _ignored: Ignored<Self>,
}
