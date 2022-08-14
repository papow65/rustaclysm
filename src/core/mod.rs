mod instruction;
mod key;
mod mesh;
mod model;
mod object_definition;
mod object_name;
mod unit;

pub use self::{
    instruction::*, key::*, mesh::*, model::*, object_definition::*, object_name::*, unit::*,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SpriteLayer {
    Front,
    Back,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Visible {
    Seen,
    Unseen,
}
