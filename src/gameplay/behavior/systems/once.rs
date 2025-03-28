use crate::gameplay::behavior::systems::core::perform_egible_character_action;
use crate::gameplay::behavior::systems::handlers::handle_action_effects;
use bevy::ecs::schedule::{IntoScheduleConfigs as _, ScheduleConfigs};
use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::{StateTransition, World};

pub(in super::super) fn behavior_systems() -> ScheduleConfigs<ScheduleSystem> {
    (
        perform_egible_character_action(),
        run_state_transitions, // only intended for PlayerActionState
        handle_action_effects(),
    )
        .chain()
}

fn run_state_transitions(world: &mut World) {
    world.run_schedule(StateTransition);
}
