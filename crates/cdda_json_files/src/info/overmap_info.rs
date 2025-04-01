use crate::{Ignored, ItemName, UntypedInfoId};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct OvermapTerrainInfo {
    pub name: ItemName,
    pub looks_like: Option<UntypedInfoId>,

    pub color: Option<Arc<str>>,
    pub connect_group: Option<Arc<str>>,
    pub delete: Option<serde_json::Value>,
    pub extend: Option<serde_json::Value>,
    pub extras: Option<Arc<str>>,
    pub flags: Option<Vec<serde_json::Value>>,
    pub id: Option<Arc<str>>,
    pub land_use_code: Option<Arc<str>>,
    pub mapgen: Option<Vec<serde_json::Value>>,
    pub mapgen_curved: Option<Vec<serde_json::Value>>,
    pub mapgen_end: Option<Vec<serde_json::Value>>,
    pub mapgen_four_way: Option<Vec<serde_json::Value>>,
    pub mapgen_straight: Option<Vec<serde_json::Value>>,
    pub mapgen_tee: Option<Vec<serde_json::Value>>,
    pub mondensity: Option<u8>,
    pub see_cost: Option<u16>,
    pub spawns: Option<serde_json::Value>,
    pub sym: Option<Arc<str>>,

    #[serde(flatten)]
    _ignored: Ignored<Self>,
}
