mod actor;
mod asset_state;
mod container;
mod damage;
mod focus;
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
mod planned_action;
mod region;
mod stamina_impact;
mod toggle;
mod type_id;
mod unit;

pub(crate) use self::{
    actor::*, asset_state::*, container::*, damage::*, focus::*, fragment::*, instruction::*,
    item::*, limited::*, mesh::*, model::*, nbor::*, object_definition::*, object_id::*, offset::*,
    planned_action::*, region::*, stamina_impact::*, toggle::*, type_id::*, unit::*,
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
