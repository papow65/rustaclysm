use crate::{Bash, Flags, HashMap, ItemName, MoveCostIncrease, ObjectId};
use serde::Deserialize;
use std::sync::{Arc, OnceLock, Weak};

#[derive(Debug, Deserialize)]
pub struct TerrainInfo {
    pub id: ObjectId,
    pub name: ItemName,
    pub move_cost: MoveCost,
    pub looks_like: Option<ObjectId>,

    /// Use [`Self.open`] where possible
    #[serde(rename(deserialize = "open"))]
    pub open_id: Option<ObjectId>,
    #[serde(skip)]
    pub open: OnceLock<Weak<TerrainInfo>>,

    /// Use [`Self.close`] where possible
    #[serde(rename(deserialize = "close"))]
    pub close_id: Option<ObjectId>,
    #[serde(skip)]
    pub close: OnceLock<Weak<TerrainInfo>>,

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
    #[must_use]
    pub const fn accessible(&self) -> bool {
        0 < self.0
    }

    #[must_use]
    pub fn value(&self) -> u8 {
        self.0.max(0) as u8
    }

    #[must_use]
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
