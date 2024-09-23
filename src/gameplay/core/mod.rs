mod asset_state;
mod container;
mod fragment;
mod instruction;
mod item;
mod limited;
mod mesh;
mod model;
mod nbor;
mod object_definition;
mod offset;
mod region;
mod type_id;
mod vision_distance;
mod walking_cost;

pub(crate) use self::asset_state::AssetState;
pub(crate) use self::container::Container;
pub(crate) use self::fragment::{Fragment, Phrase, Positioning};
pub(crate) use self::instruction::*;
pub(crate) use self::item::{Item, ItemItem};
pub(crate) use self::limited::{Evolution, Limited};
pub(crate) use self::mesh::MeshInfo;
pub(crate) use self::model::{Layers, Model, ModelShape, SpriteOrientation, Transform2d};
pub(crate) use self::nbor::{CardinalDirection, HorizontalDirection, Nbor, NborDistance};
pub(crate) use self::object_definition::ObjectCategory;
pub(crate) use self::offset::{LevelOffset, PosOffset};
pub(crate) use self::region::{Region, ZoneRegion};
pub(crate) use self::type_id::TypeId;
pub(crate) use self::vision_distance::VisionDistance;
pub(crate) use self::walking_cost::WalkingCost;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum SpriteLayer {
    Front,
    Back,
}

/// Visible to the player character
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Visible {
    Seen,
    Unseen,
}
