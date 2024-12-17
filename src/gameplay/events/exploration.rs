use crate::gameplay::Pos;
use bevy::prelude::Event;

/// The player has seen the pos
#[derive(Debug, Event)]
pub(crate) struct Exploration(Pos);

impl Exploration {
    pub(crate) const fn new(pos: Pos) -> Self {
        Self(pos)
    }

    pub(crate) const fn pos(&self) -> Pos {
        self.0
    }
}
