use crate::{CommonItemInfo, Flags, Ignored, InfoId, VehiclePartInfo};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct ItemMigration {
    pub id: InfoId<CommonItemInfo>,
    pub replace: InfoId<CommonItemInfo>,

    pub charges: Option<u16>,
    pub contents: Option<Vec<serde_json::Value>>,
    pub flags: Flags,
    pub reset_item_vars: Option<bool>,
    pub sealed: Option<bool>,
    pub variant: Option<Arc<str>>,

    #[serde(flatten)]
    _ignored: Ignored<Self>,
}

#[derive(Debug, Deserialize)]
pub struct VehiclePartMigration {
    pub from: InfoId<VehiclePartInfo>,
    pub to: InfoId<VehiclePartInfo>,

    #[serde(flatten)]
    _ignored: Ignored<Self>,
}
