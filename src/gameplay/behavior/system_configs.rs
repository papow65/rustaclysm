use crate::common::log_if_slow;
use crate::gameplay::behavior::schedule::BehaviorSchedule;
use crate::gameplay::behavior::systems::core::{
    egible_character, perform_action, plan_action, proces_impact,
};
use crate::gameplay::behavior::systems::handlers::{
    combine_items, spawn_broken_terrain, toggle_doors, update_corpses, update_damaged_characters,
    update_damaged_corpses, update_damaged_terrain, update_explored, update_healed_characters,
};
use crate::gameplay::behavior::systems::refresh::{
    check_items, update_hidden_item_visibility, update_peeking_transforms, update_transforms,
    update_visualization_on_player_move, update_visualization_on_weather_change,
};
use crate::gameplay::systems::update_visualization_on_item_move;
use crate::gameplay::{
    ActorEvent, CorpseEvent, Damage, Healing, InstructionQueue, PlayerActionState,
    RefreshAfterBehavior, RelativeSegments, TerrainEvent, Toggle,
};
use bevy::ecs::system::SystemState;
use bevy::prelude::{
    on_event, resource_exists, resource_exists_and_changed, IntoSystem, IntoSystemConfigs, Res,
    State, StateTransition, World,
};
use std::time::{Duration, Instant};

pub(super) fn loop_behavior_and_refresh() -> impl IntoSystemConfigs<()> {
    (
        loop_behavior,
        (
            update_transforms,
            update_peeking_transforms
                .run_if(resource_exists_and_changed::<State<PlayerActionState>>),
            update_hidden_item_visibility,
            update_visualization_on_item_move,
            update_visualization_on_player_move,
            update_visualization_on_weather_change,
            check_items,
        )
            .run_if(on_event::<RefreshAfterBehavior>),
    )
        .chain()
        .run_if(resource_exists::<RelativeSegments>)
}

/// This repeatedly runs [`BehaviorSchedule`], until the time runs out or player input is required.
fn loop_behavior(world: &mut World) {
    let start = Instant::now();

    let max_time = if world
        .get_resource::<State<PlayerActionState>>()
        .is_some_and(|state| {
            matches!(
                **state,
                PlayerActionState::Crafting { .. }
                    | PlayerActionState::Waiting { .. }
                    | PlayerActionState::Sleeping { .. }
            )
        }) {
        // Allows 10 fps
        Duration::from_secs_f32(0.09)
    } else {
        // Allows 120 fps
        Duration::from_secs_f32(0.005)
    };

    let mut count = 0;
    while !waiting_for_user_input(world) {
        world.run_schedule(BehaviorSchedule);
        count += 1;
        if max_time < start.elapsed() {
            println!(
                "run_behavior_schedule could only handle {count} iterations before the timeout"
            );
            break;
        }
    }

    if 0 < count {
        world.send_event(RefreshAfterBehavior);
    }

    log_if_slow("run_behavior_schedule", start);
}

/// All NPC mave a timeout and the player has an empty instruction queue
fn waiting_for_user_input(world: &mut World) -> bool {
    let mut system_state = SystemState::<Res<InstructionQueue>>::new(world);
    let instruction_queue = system_state.get(world);
    instruction_queue.is_waiting()
}

pub(super) fn behavior_systems() -> impl IntoSystemConfigs<()> {
    (
        egible_character
            .pipe(plan_action)
            .pipe(perform_action)
            .pipe(proces_impact),
        run_state_transitions, // only intended for PlayerActionState
        (
            (
                // actor events
                // Make sure killed actors are handled early
                update_damaged_characters.run_if(on_event::<ActorEvent<Damage>>),
                (
                    update_healed_characters.run_if(on_event::<ActorEvent<Healing>>),
                    update_corpses,
                    update_explored,
                ),
            )
                .chain(),
            (
                // item events
                update_damaged_corpses.run_if(on_event::<CorpseEvent<Damage>>),
                combine_items,
            )
                .chain(),
            (
                // terrain events
                // Make sure destoyed items are handled early
                update_damaged_terrain
                    .pipe(spawn_broken_terrain)
                    .run_if(on_event::<TerrainEvent<Damage>>),
                toggle_doors.run_if(on_event::<TerrainEvent<Toggle>>),
            )
                .chain(),
        ),
    )
        .chain()
}

fn run_state_transitions(world: &mut World) {
    world.run_schedule(StateTransition);
}
