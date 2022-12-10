mod instruction;
mod item_info;
mod key;
mod mesh;
mod model;
mod object_definition;
mod object_name;
mod region;
mod unit;

pub(crate) use self::{
    instruction::*, item_info::*, key::*, mesh::*, model::*, object_definition::*, object_name::*,
    region::*, unit::*,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum SpriteLayer {
    Front,
    Back,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Visible {
    Seen,
    Unseen,
}
