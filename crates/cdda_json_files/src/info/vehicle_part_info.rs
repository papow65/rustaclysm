use crate::{CommonItemInfo, InfoId, RequiredLinkedLater, UntypedInfoId};
use crate::{Flags, ItemName};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VehiclePartInfo {
    pub id: InfoId<Self>,
    pub name: Option<ItemName>,
    pub item: RequiredLinkedLater<CommonItemInfo>,
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
