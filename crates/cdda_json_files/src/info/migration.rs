use crate::{CommonItemInfo, InfoId, VehiclePartInfo};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ItemMigration {
    pub id: InfoId<CommonItemInfo>,
    pub replace: InfoId<CommonItemInfo>,
}

#[derive(Debug, Deserialize)]
pub struct VehiclePartMigration {
    pub from: InfoId<VehiclePartInfo>,
    pub to: InfoId<VehiclePartInfo>,
}
