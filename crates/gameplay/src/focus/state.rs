use crate::{CancelHandling, PlayerActionState, Pos, ZoneLevel};
use application_state::ApplicationState;
use bevy::prelude::{StateSet as _, SubStates};
use std::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, SubStates)]
#[source(ApplicationState = ApplicationState::Gameplay)]
pub(crate) enum FocusState {
    #[default]
    Normal,
    ExaminingPos(Pos),
    ExaminingZoneLevel(ZoneLevel),
}

impl FocusState {
    pub(crate) const fn cancel_handling(
        &self,
        player_action_state: &PlayerActionState,
    ) -> CancelHandling {
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
