use crate::{CommonItemInfo, InfoId, UntypedInfoId};
use crate::{Flags, ItemName};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VehiclePartInfo {
    pub name: Option<ItemName>,
    pub item: InfoId<CommonItemInfo>,
    pub looks_like: Option<UntypedInfoId>,
    pub flags: Flags,
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
