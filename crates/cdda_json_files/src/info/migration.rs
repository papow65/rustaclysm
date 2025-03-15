use crate::InfoId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ItemMigration {
    pub id: InfoId,
    pub replace: InfoId,
}

#[derive(Debug, Deserialize)]
pub struct VehiclePartMigration {
    pub from: InfoId,
    pub to: InfoId,
}
