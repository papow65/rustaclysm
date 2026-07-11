use crate::actor::messages::{NoTargetsNearby, NothingToCloseNearby};
use crate::actor::player::PlayerActionState;
use crate::{PlannedAction, QueuedInstruction};
use bevy::prelude::{NextState, ResMut};
use gameplay_location::{Nbor, Pos};
use gameplay_log::LogMessageWriter;

pub(in super::super) fn handle_attack(
    next_state: &mut ResMut<NextState<PlayerActionState>>,
    message_writer: &mut LogMessageWriter,
    envir: &crate::Envir,
    pos: Pos,
) -> Option<PlannedAction> {
    let attackable_nbors = envir
        .nbors_for_exploring(pos, &QueuedInstruction::Attack)
        .collect::<Vec<_>>();
    match attackable_nbors.len() {
        0 => {
            message_writer.send(NoTargetsNearby);
            None
        }
        1 => Some(PlannedAction::attack(attackable_nbors[0])),
        _ => {
            next_state.set(PlayerActionState::PickingNbor(
                crate::actor::player::PickingNbor::Attacking,
            ));
            None
        }
    }
}

pub(in super::super) fn handle_smash(
    next_state: &mut ResMut<NextState<PlayerActionState>>,
    message_writer: &mut LogMessageWriter,
    envir: &crate::Envir,
    pos: Pos,
) -> Option<PlannedAction> {
    let smashable_nbors = envir
        .nbors_for_exploring(pos, &QueuedInstruction::Smash)
        .collect::<Vec<_>>();
    match smashable_nbors.len() {
        0 => {
            message_writer.send(NoTargetsNearby);
            None
        }
        1 => Some(PlannedAction::smash(smashable_nbors[0])),
        _ => {
            next_state.set(PlayerActionState::PickingNbor(
                crate::actor::player::PickingNbor::Smashing,
            ));
            None
        }
    }
}

pub(in super::super) fn handle_pulp(
    next_state: &mut ResMut<NextState<PlayerActionState>>,
    message_writer: &mut LogMessageWriter,
    envir: &crate::Envir,
    pos: Pos,
) -> Option<PlannedAction> {
    let pulpable_nbors = envir
        .nbors_for_exploring(pos, &QueuedInstruction::Pulp)
        .filter_map(|nbor| {
            if let Nbor::Horizontal(horizontal) = nbor {
                Some(horizontal)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    //trace!("Pulping {} targets", pulpable_nbors.len());
    match pulpable_nbors.len() {
        0 => {
            message_writer.send(NoTargetsNearby);
            None
        }
        1 => {
            //trace!("Pulping target found -> active");
            next_state.set(PlayerActionState::Pulping {
                direction: pulpable_nbors[0],
            });
            Some(PlannedAction::pulp(pulpable_nbors[0]))
        }
        _ => {
            //trace!("Pulping choice -> inactive");
            next_state.set(PlayerActionState::PickingNbor(
                crate::actor::player::PickingNbor::Pulping,
            ));
            None
        }
    }
}

pub(in super::super) fn handle_close(
    next_state: &mut ResMut<NextState<PlayerActionState>>,
    message_writer: &mut LogMessageWriter,
    envir: &crate::Envir,
    pos: Pos,
) -> Option<PlannedAction> {
    let closable_nbors = envir
        .nbors_for_exploring(pos, &QueuedInstruction::Close)
        .filter_map(|nbor| {
            if let Nbor::Horizontal(horizontal) = nbor {
                Some(horizontal)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    match closable_nbors.len() {
        0 => {
            message_writer.send(NothingToCloseNearby);
            None
        }
        1 => Some(PlannedAction::close(closable_nbors[0])),
        _ => {
            next_state.set(PlayerActionState::PickingNbor(
                crate::actor::player::PickingNbor::Closing,
            ));
            None
        }
    }
}
