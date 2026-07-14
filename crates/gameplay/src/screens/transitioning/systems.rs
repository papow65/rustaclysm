use crate::GameplayScreenState;
use bevy::prelude::{NextState, ResMut};

pub(super) fn to_base_screen(
    mut next_gameplay_screen_state: ResMut<NextState<GameplayScreenState>>,
) {
    next_gameplay_screen_state.set(GameplayScreenState::Base);
}
