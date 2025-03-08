use crate::{HashMap, ItemName, ObjectId};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct Quality {
    pub id: ObjectId,

    pub name: ItemName,

    #[serde(default)]
    pub usages: Vec<(u8, Vec<Arc<str>>)>,

    #[serde(flatten)]
    pub extra: HashMap<Arc<str>, serde_json::Value>,
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
