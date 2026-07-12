use application_state::ApplicationState;
use bevy::prelude::SubStates;
use gameplay_location::{Pos, ZoneLevel};
use gameplay_player::PlayerActionState;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancelHandling {
    Queued,
    Menu,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, SubStates)]
#[source(ApplicationState = ApplicationState::Gameplay)]
pub enum FocusState {
    #[default]
    Normal,
    ExaminingPos(Pos),
    ExaminingZoneLevel(ZoneLevel),
}

impl FocusState {
    #[must_use]
    pub const fn cancel_handling(&self, player_action_state: &PlayerActionState) -> CancelHandling {
        if !matches!(*self, Self::Normal) {
            CancelHandling::Queued
        } else if matches!(
            player_action_state,
            PlayerActionState::Normal | PlayerActionState::Sleeping { .. }
        ) {
            CancelHandling::Menu
        } else {
            CancelHandling::Queued
        }
    }
}

impl fmt::Display for FocusState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            Self::Normal => "",
            Self::ExaminingPos(_) => "Examining",
            Self::ExaminingZoneLevel(_) => "Examining map",
        })
    }
}
