use crate::{Flags, Ignored, InfoId, ItemName, UntypedInfoId};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct OvermapTerrainInfo {
    pub id: InfoId<Self>,
    pub name: ItemName,
    pub color: Arc<str>,

    pub looks_like: Option<UntypedInfoId>,
    pub connect_group: Option<Arc<str>>,
    pub delete: Option<JsonValue>,
    pub extend: Option<JsonValue>,
    pub extras: Option<Arc<str>>,
    pub flags: Flags,
    pub land_use_code: Option<Arc<str>>,
    pub mapgen: Option<Vec<JsonValue>>,
    pub mapgen_curved: Option<Vec<JsonValue>>,
    pub mapgen_end: Option<Vec<JsonValue>>,
    pub mapgen_four_way: Option<Vec<JsonValue>>,
    pub mapgen_straight: Option<Vec<JsonValue>>,
    pub mapgen_tee: Option<Vec<JsonValue>>,
    pub mondensity: Option<u8>,
    pub see_cost: Option<u16>,
    pub spawns: Option<JsonValue>,
    pub sym: Option<Arc<str>>,

    #[serde(flatten)]
    _ignored: Ignored<Self>,
}
