use crate::cdda::ItemName;
use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Quality {
    pub(crate) name: ItemName,

    #[expect(unused)]
    #[serde(default)]
    pub(crate) usages: Vec<(u8, Vec<String>)>,

    #[expect(unused)]
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
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
