use crate::gameplay::ObjectId;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Spawn {
    pub(crate) id: ObjectId,

    #[expect(unused)]
    count: i32,

    pub(crate) x: i32,
    pub(crate) z: i32,

    #[expect(unused)]
    faction_id: i32,

    #[expect(unused)]
    mission_id: i32,

    #[expect(unused)]
    friendly: bool,

    #[expect(unused)]
    name: Option<String>,
}
