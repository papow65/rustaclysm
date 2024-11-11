mod asset_state;
mod instruction;
mod limited;
mod local_terrain;
mod nbor;
mod offset;
mod region;
mod vision_distance;
mod walking_cost;

pub(crate) use self::asset_state::AssetState;
pub(crate) use self::instruction::*;
pub(crate) use self::limited::{Evolution, Limited};
pub(crate) use self::local_terrain::LocalTerrain;
pub(crate) use self::nbor::{CardinalDirection, HorizontalDirection, Nbor, NborDistance};
pub(crate) use self::offset::{LevelOffset, PosOffset};
pub(crate) use self::region::{Region, ZoneRegion};
pub(crate) use self::vision_distance::VisionDistance;
pub(crate) use self::walking_cost::WalkingCost;

/// Visible to the player character
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Visible {
    Seen,
    Unseen,
}
