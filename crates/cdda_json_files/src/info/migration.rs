use crate::ObjectId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ItemMigration {
    pub replace: ObjectId,
}

#[derive(Debug, Deserialize)]
pub struct VehiclePartMigration {
    pub replace: ObjectId,
}
