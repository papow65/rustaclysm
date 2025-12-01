mod limited;
mod local_terrain;
mod region;
mod walking_cost;

pub(crate) use self::limited::{Evolution, Limited};
pub(crate) use self::local_terrain::LocalTerrain;
pub(crate) use self::region::{Region, ZoneRegion};
pub(crate) use self::walking_cost::WalkingCost;

/// Visible to the player character
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Visible {
    Seen,
    Unseen,
}
