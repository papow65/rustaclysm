use crate::cdda::{Flags, ItemName};
use crate::gameplay::ObjectId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct VehiclePartInfo {
    pub(crate) name: Option<ItemName>,
    pub(crate) item: ObjectId,
    pub(crate) looks_like: Option<ObjectId>,
    pub(crate) flags: Flags,
}

#[cfg(test)]
mod item_tests {
    use super::*;
    #[test]
    fn it_works() {
        let json = include_str!("test_train_motor.json");
        let result = serde_json::from_str::<VehiclePartInfo>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
