use crate::ObjectId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ItemMigration {
    pub id: ObjectId,
    pub replace: ObjectId,
}

#[derive(Debug, Deserialize)]
pub struct VehiclePartMigration {
    pub from: ObjectId,
    pub to: ObjectId,
}
