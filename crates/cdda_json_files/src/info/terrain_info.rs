use crate::ObjectId;
use crate::{Bash, Flags, ItemName, MoveCostIncrease};
use bevy::utils::HashMap;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct TerrainInfo {
    pub name: ItemName,
    pub move_cost: MoveCost,
    pub looks_like: Option<ObjectId>,
    pub open: Option<ObjectId>,
    pub close: Option<ObjectId>,
    pub flags: Flags,
    pub bash: Option<Bash>,

    #[expect(unused)]
    #[serde(flatten)]
    extra: HashMap<Arc<str>, serde_json::Value>,
}

// TODO What does a negative value mean?
/// 0 -> inaccessible
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, PartialOrd)]
pub struct MoveCost(i8);

impl MoveCost {
    pub const fn accessible(&self) -> bool {
        0 < self.0
    }

    pub fn value(&self) -> u8 {
        self.0.max(0) as u8
    }

    pub fn adjust(&self, cost_mod: Option<MoveCostIncrease>) -> Self {
        let extra = cost_mod.map_or(0, |c| c.0);
        Self(self.0 + extra as i8)
    }
}

impl Default for MoveCost {
    fn default() -> Self {
        Self(2)
    }
}
