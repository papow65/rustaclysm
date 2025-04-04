use crate::{Bash, Flags, InfoId, ItemName, MoveCostIncrease, OptionalLinkedLater, UntypedInfoId};
use bevy_platform_support::collections::HashMap;
use serde::Deserialize;
use std::sync::{Arc, LazyLock};

#[derive(Debug, Deserialize)]
pub struct TerrainInfo {
    pub id: InfoId<Self>,
    pub name: ItemName,
    pub move_cost: MoveCost,
    pub looks_like: Option<UntypedInfoId>,

    pub open: OptionalLinkedLater<TerrainInfo>,
    pub close: OptionalLinkedLater<TerrainInfo>,

    pub flags: Flags,
    pub bash: Option<Bash>,

    #[expect(unused)]
    #[serde(flatten)]
    extra: HashMap<Arc<str>, serde_json::Value>,
}

impl TerrainInfo {
    pub fn is_similar(&self, other: &Self) -> bool {
        static PAVEMENT: LazyLock<InfoId<TerrainInfo>> =
            LazyLock::new(|| InfoId::new("t_pavement"));
        static PAVEMENT_DOT: LazyLock<InfoId<TerrainInfo>> =
            LazyLock::new(|| InfoId::new("t_pavement_y"));

        self.id == other.id
            || (self.id == *PAVEMENT && other.id == *PAVEMENT_DOT)
            || (self.id == *PAVEMENT_DOT && other.id == *PAVEMENT)
    }
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
