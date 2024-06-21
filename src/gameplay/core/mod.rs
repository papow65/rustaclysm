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
mod unit;
mod vision_distance;

pub(crate) use self::{
    asset_state::*, container::*, fragment::*, instruction::*, item::*, limited::*, mesh::*,
    model::*, nbor::*, object_definition::*, object_id::*, offset::*, region::*, toggle::*,
    type_id::*, unit::*, vision_distance::*,
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
