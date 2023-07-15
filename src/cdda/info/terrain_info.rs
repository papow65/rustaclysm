use crate::prelude::{Bash, Flags, ItemName, MoveCostIncrease, ObjectId};
use bevy::utils::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct TerrainInfo {
    pub(crate) name: ItemName,
    pub(crate) move_cost: MoveCost,
    pub(crate) looks_like: Option<ObjectId>,
    pub(crate) open: Option<ObjectId>,
    pub(crate) close: Option<ObjectId>,
    pub(crate) flags: Flags,
    pub(crate) bash: Option<Bash>,

    #[allow(unused)]
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, PartialOrd)]
pub(crate) struct MoveCost(pub(crate) u8);

impl MoveCost {
    pub(crate) fn adjust(&self, cost_mod: Option<MoveCostIncrease>) -> Self {
        let extra = cost_mod.map_or(0, |c| c.0);
        Self(self.0 + extra)
    }
}

impl Default for MoveCost {
    fn default() -> Self {
        Self(2)
    }
}
