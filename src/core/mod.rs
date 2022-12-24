mod instruction;
mod key;
mod mesh;
mod model;
mod nbor;
mod object_definition;
mod object_id;
mod offset;
mod region;
mod type_id;
mod unit;

pub(crate) use self::{
    instruction::*, key::*, mesh::*, model::*, nbor::*, object_definition::*, object_id::*,
    offset::*, region::*, type_id::*, unit::*,
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
