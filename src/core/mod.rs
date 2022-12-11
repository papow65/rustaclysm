mod instruction;
mod item_info;
mod key;
mod mesh;
mod model;
mod nbor;
mod object_definition;
mod object_name;
mod offset;
mod region;
mod unit;

pub(crate) use self::{
    instruction::*, item_info::*, key::*, mesh::*, model::*, nbor::*, object_definition::*,
    object_name::*, offset::*, region::*, unit::*,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum SpriteLayer {
    Front,
    Back,
}

/** Visible to the player character */
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Visible {
    Seen,
    Unseen,
}
