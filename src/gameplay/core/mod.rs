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
mod object_id;
mod offset;
mod region;
mod toggle;
mod type_id;
mod vision_distance;

pub(crate) use self::{
    asset_state::AssetState,
    container::Container,
    fragment::{Fragment, Phrase, Positioning},
    instruction::*,
    item::{Item, ItemItem},
    limited::{Evolution, Limited},
    mesh::MeshInfo,
    model::{Layers, Model, ModelShape, SpriteOrientation, Transform2d},
    nbor::{CardinalDirection, HorizontalDirection, Nbor, NborDistance},
    object_definition::ObjectCategory,
    object_id::ObjectId,
    offset::{LevelOffset, PosOffset},
    region::{Region, ZoneRegion},
    toggle::Toggle,
    type_id::TypeId,
    vision_distance::VisionDistance,
};

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
