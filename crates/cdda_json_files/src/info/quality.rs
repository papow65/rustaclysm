use crate::{Ignored, InfoId, ItemAction, ItemName, RequiredLinkedLater};
use serde::Deserialize;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

// PartialEq, Eq, and Hash manually implemented below
/// Do not confuse with [`ItemQuality`]
#[derive(Debug, Deserialize)]
pub struct Quality {
    pub id: InfoId<Self>,
    pub name: ItemName,

    #[serde(default)]
    pub usages: Vec<(u8, Vec<RequiredLinkedLater<ItemAction>>)>,

    #[serde(flatten)]
    _ignored: Ignored<Self>,
}

impl PartialEq for Quality {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Quality {}

impl Hash for Quality {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[derive(Debug, Deserialize)]
pub struct ItemQuality {
    pub id: RequiredLinkedLater<Quality>,
    pub level: i8,
}

impl ItemQuality {
    #[track_caller]
    pub fn as_tuple(&self) -> Option<(Arc<Quality>, i8)> {
        self.id.get_option().map(|id| (id, self.level))
    }
}

#[cfg(test)]
mod quality_tests {
    use super::*;
    use serde_json::from_str as from_json_str;

    #[test]
    fn it_works() {
        let json = include_str!("test_lockpick.json");
        let result = from_json_str::<Quality>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
