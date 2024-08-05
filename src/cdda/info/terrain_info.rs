use crate::cdda::{Bash, Flags, ItemName, MoveCostIncrease};
use crate::gameplay::ObjectId;
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

// TODO What does a negative value mean?
/// 0 -> inaccessible
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, PartialOrd)]
pub(crate) struct MoveCost(i8);

impl MoveCost {
    pub(crate) const fn accessible(&self) -> bool {
        0 < self.0
    }

    pub(crate) fn value(&self) -> u8 {
        self.0.max(0) as u8
    }

    pub(crate) fn adjust(&self, cost_mod: Option<MoveCostIncrease>) -> Self {
        let extra = cost_mod.map_or(0, |c| c.0);
        Self(self.0 + extra as i8)
    }
}

impl Default for MoveCost {
    fn default() -> Self {
        Self(2)
    }
}
