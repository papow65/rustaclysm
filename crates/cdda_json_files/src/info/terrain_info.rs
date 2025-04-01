use crate::{
    Bash, Flags, Ignored, InfoId, ItemName, MoveCostIncrease, OptionalLinkedLater, UntypedInfoId,
};
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

    pub allowed_template_ids: Option<Vec<serde_json::Value>>,
    pub boltcut: Option<serde_json::Value>,
    pub color: Option<serde_json::Value>,
    pub comfort: Option<u8>,
    pub connect_groups: Option<serde_json::Value>,
    pub connects_to: Option<Arc<str>>,
    pub coverage: Option<u8>,
    pub curtain_transform: Option<Arc<str>>,
    pub deconstruct: Option<serde_json::Value>,
    pub description: Option<Arc<str>>,
    pub emissions: Option<Vec<serde_json::Value>>,
    pub examine_action: Option<serde_json::Value>,
    pub floor_bedding_warmth: Option<u16>,
    pub hacksaw: Option<serde_json::Value>,
    pub harvest_by_season: Option<Vec<serde_json::Value>>,
    pub heat_radiation: Option<u8>,
    pub light_emitted: Option<u8>,
    pub lockpick_message: Option<Arc<str>>,
    pub lockpick_result: Option<Arc<str>>,
    pub max_volume: Option<Arc<str>>,
    pub oxytorch: Option<serde_json::Value>,
    pub prying: Option<serde_json::Value>,
    pub roof: Option<Arc<str>>,
    pub rotates_to: Option<Arc<str>>,
    pub shoot: Option<serde_json::Value>,
    pub symbol: Option<Arc<str>>,
    pub transforms_into: Option<Arc<str>>,
    pub trap: Option<Arc<str>>,

    #[serde(flatten)]
    _ignored: Ignored<Self>,
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
    pub const fn accessible(self) -> bool {
        0 < self.0
    }

    #[must_use]
    pub fn value(self) -> u8 {
        self.0.max(0) as u8
    }

    #[must_use]
    pub fn adjust(self, cost_mod: Option<MoveCostIncrease>) -> Self {
        let extra = cost_mod.map_or(0, |c| c.0);
        Self(self.0 + extra as i8)
    }
}

impl Default for MoveCost {
    fn default() -> Self {
        Self(2)
    }
}
