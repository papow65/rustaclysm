use bevy::prelude::Resource;
use std::num::Wrapping;

pub(super) type GameplayCount = Wrapping<u32>;

#[derive(Debug, Default, Resource)]
pub(super) struct GameplayCounter(GameplayCount);

impl GameplayCounter {
    pub(super) const fn get(&self) -> GameplayCount {
        self.0
    }

    pub(super) fn increase(&mut self) {
        self.0 += 1;
    }
}
