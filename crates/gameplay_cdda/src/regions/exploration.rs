use bevy::prelude::Message;
use gameplay_location::{Overzone, Pos, SubzoneLevel, ZoneLevel};

#[derive(Message)]
pub enum Exploration {
    /// The player has seen the pos
    Pos(Pos),
    /// The player has seen given pos in the subzone level
    SubzoneLevel(SubzoneLevel, Vec<Pos>),
    /// The player has seen given zone levels in the overzone
    Overzone(Overzone, Vec<ZoneLevel>),
}
