use crate::{HashMap, InfoId, ItemName};
use serde::Deserialize;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

// PartialEq, Eq, and Hash manually implemented below
#[derive(Debug, Deserialize)]
pub struct Quality {
    pub id: InfoId,
    pub name: ItemName,

    #[serde(default)]
    pub usages: Vec<(u8, Vec<Arc<str>>)>,

    #[serde(flatten)]
    pub extra: HashMap<Arc<str>, serde_json::Value>,
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

#[cfg(test)]
mod quality_tests {
    use super::*;
    #[test]
    fn it_works() {
        let json = include_str!("test_lockpick.json");
        let result = serde_json::from_str::<Quality>(json);
        assert!(result.is_ok(), "{result:?}");
    }
}
