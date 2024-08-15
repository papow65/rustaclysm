use crate::gameplay::ObjectId;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Spawn {
    pub(crate) id: ObjectId,

    #[allow(unused)]
    count: i32,

    pub(crate) x: i32,
    pub(crate) z: i32,

    #[allow(unused)]
    faction_id: i32,

    #[allow(unused)]
    mission_id: i32,

    #[allow(unused)]
    friendly: bool,

    #[allow(unused)]
    name: Option<String>,
}
