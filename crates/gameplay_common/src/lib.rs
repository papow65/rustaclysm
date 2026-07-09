mod limited;
mod local_terrain;
mod region;
mod walking_cost;

pub use self::limited::{Evolution, Limited};
pub use self::local_terrain::LocalTerrain;
pub use self::region::{Region, ZoneRegion};
pub use self::walking_cost::WalkingCost;

/// Visible to the player character
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visible {
    Seen,
    Unseen,
}
