use crate::{CharacterInfo, RequiredLinkedLater};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Character {
    #[serde(rename = "id")]
    pub info: RequiredLinkedLater<CharacterInfo>,

    #[expect(unused)]
    count: i32,

    pub x: i32,
    pub z: i32,

    #[expect(unused)]
    faction_id: i32,

    #[expect(unused)]
    mission_id: i32,

    #[expect(unused)]
    friendly: bool,

    #[expect(unused)]
    name: Option<Arc<str>>,
}
