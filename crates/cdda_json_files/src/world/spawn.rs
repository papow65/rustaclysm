use crate::ObjectId;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spawn {
    pub id: ObjectId,

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
