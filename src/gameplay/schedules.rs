use crate::prelude::*;
use bevy::{
    ecs::{schedule::ScheduleLabel, system::SystemState},
    prelude::*,
};
use std::time::{Duration, Instant};

/** This is only run when the game when any character acts, sometimes multiple times per tick. */
#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct BehaviorSchedule;

pub(crate) fn create_behavior_schedule(app: &mut App) {
    app.init_schedule(BehaviorSchedule);

    app.add_systems(
        BehaviorSchedule,
        (
            egible_character
                .pipe(plan_action)
                .pipe(perform_action)
                .pipe(proces_impact),
            (
                (
                    // actor events
                    // Make sure killed actors are handled early
                    update_damaged_characters.run_if(on_event::<ActorEvent<Damage>>()),
                    (
                        update_stamina.run_if(on_event::<ActorEvent<StaminaImpact>>()),
                        update_healed_characters.run_if(on_event::<ActorEvent<Healing>>()),
                        update_corpses,
                        update_explored,
                    ),
                )
                    .chain(),
                (
                    // item events
                    update_damaged_corpses.run_if(on_event::<CorpseEvent<Damage>>()),
                    combine_items,
                )
                    .chain(),
                (
                    // terrain events
                    // Make sure destoyed items are handled early
                    update_damaged_terrain.run_if(on_event::<TerrainEvent<Damage>>()),
                    toggle_doors.run_if(on_event::<TerrainEvent<Toggle>>()),
                )
                    .chain(),
            ),
        )
            .chain(),
    );
}

pub(crate) fn behavior_systems() -> impl IntoSystemConfigs<()> {
    (
        run_behavior_schedule,
        (
            update_transforms,
            update_hidden_item_visibility,
            update_cursor_visibility_on_player_change,
            update_visualization_on_item_move,
            update_visualization_on_focus_move,
            update_visualization_on_weather_change.run_if(resource_exists_and_changed::<Timeouts>),
            update_camera_base.run_if(resource_exists_and_changed::<PlayerActionState>),
            #[cfg(debug_assertions)]
            check_items,
            #[cfg(feature = "log_archetypes")]
            list_archetypes,
        )
            .run_if(on_event::<RefreshAfterBehavior>()),
    )
        .chain()
}

fn run_behavior_schedule(world: &mut World) {
    let start = Instant::now();

    let max_time =
        if let Some(PlayerActionState::Waiting { .. } | PlayerActionState::Sleeping { .. }) =
            world.get_resource::<PlayerActionState>()
        {
            // Allows 10 fps
            Duration::from_secs_f32(0.09)
        } else {
            // Allows 120 fps
            Duration::from_secs_f32(0.005)
        };

    let mut count = 0;
    let mut over_time = false;
    while !waiting_for_user_input(world) && !over_time {
        world.run_schedule(BehaviorSchedule);
        count += 1;
        over_time = max_time < start.elapsed();
    }

    if over_time {
        println!("run_behavior_schedule could only handle {count} iterations before the timeout");
    }

    log_if_slow("run_behavior_schedule", start);

    if 0 < count {
        world.send_event(RefreshAfterBehavior);
    }
}

/** All NPC mave a timeout and the player has an empty instruction queue */
fn waiting_for_user_input(world: &mut World) -> bool {
    let mut system_state = SystemState::<(Res<InstructionQueue>,)>::new(world);
    let (instruction_queue,) = system_state.get(world);
    instruction_queue.is_waiting()
}
