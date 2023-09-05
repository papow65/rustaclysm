use crate::prelude::*;
use bevy::{
    ecs::{schedule::ScheduleLabel, system::SystemState},
    prelude::*,
};
use std::time::{Duration, Instant};

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
                .pipe(handle_impact),
            apply_deferred,
            (
                (
                    update_damaged_characters,
                    apply_deferred,
                    update_healed_characters,
                )
                    .chain(),
                toggle_doors,
                (update_damaged_items, combine_items).chain(),
            ),
            apply_deferred,
            (
                update_transforms,
                update_hidden_item_visibility,
                update_cursor_visibility_on_player_change,
                update_visualization_on_item_move,
                update_visualization_on_focus_move,
                update_visualization_on_weather_change
                    .run_if(resource_exists_and_changed::<Timeouts>()),
                update_camera_base.run_if(resource_exists_and_changed::<PlayerActionState>()),
            ),
        )
            .chain()
            .run_if(in_state(ApplicationState::Gameplay)),
    );
}

pub(crate) fn run_behavior_schedule(world: &mut World) {
    let start = Instant::now();

    let mut count = 0;
    while !waiting_for_user_input(world) && !over_time(&start, count) {
        world.run_schedule(BehaviorSchedule);
        count += 1;
    }

    log_if_slow("run_behavior_schedule", start);
}

/** All NPC mave a timeout and the player has an empty instruction queue */
fn waiting_for_user_input(world: &mut World) -> bool {
    let mut system_state = SystemState::<(Res<InstructionQueue>,)>::new(world);
    let (instruction_queue,) = system_state.get(world);
    instruction_queue.is_waiting()
}

fn over_time(start: &Instant, count: usize) -> bool {
    let over_time = Duration::from_millis(2) * 3 / 4 < start.elapsed();
    if over_time {
        eprintln!("run_behavior_schedule could ony handle {count} iterations before the timeout");
    }
    over_time
}
