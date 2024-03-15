use crate::prelude::{CancelHandling, PlayerActionState, Pos, ZoneLevel};
use bevy::prelude::States;
use std::fmt;

/** Conceptually, this is a child state of `GameplayScreenState::Base` */
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash, States)]
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
