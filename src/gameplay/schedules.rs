use crate::prelude::*;
use bevy::{
    ecs::{schedule::ScheduleLabel, system::SystemState},
    prelude::*,
};
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq)]
pub(crate) enum DisplayHandling {
    Refresh,
    Keep,
}

impl From<usize> for DisplayHandling {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Keep,
            _ => Self::Refresh,
        }
    }
}

/** This is only run when the game when any character acts, sometimes multiple times per tick. */
#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct BehaviorSchedule;

/** This is run after the behavior schedule, but no more than once per tick. */
#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct DisplayBehaviorSchedule;

pub(crate) fn create_schedules(app: &mut App) {
    create_behavior_schedule(app);
    create_display_behavior_schedule(app);
}
pub(crate) fn create_behavior_schedule(app: &mut App) {
    app.init_schedule(BehaviorSchedule);

    app.add_systems(
        BehaviorSchedule,
        (
            egible_character.pipe(plan_action).pipe(send_action_event),
            (
                check_action_plan_amount,
                single_action
                    .pipe(perform_stay)
                    .pipe(proces_impact)
                    .run_if(on_event::<ActionEvent<Stay>>()),
                single_action
                    .pipe(perform_step)
                    .pipe(proces_impact)
                    .run_if(on_event::<ActionEvent<Step>>()),
                single_action
                    .pipe(perform_attack)
                    .pipe(proces_impact)
                    .run_if(on_event::<ActionEvent<Attack>>()),
                single_action
                    .pipe(perform_smash)
                    .pipe(proces_impact)
                    .run_if(on_event::<ActionEvent<Smash>>()),
                single_action
                    .pipe(perform_close)
                    .pipe(proces_impact)
                    .run_if(on_event::<ActionEvent<Close>>()),
                single_action
                    .pipe(perform_wield)
                    .pipe(proces_impact)
                    .run_if(on_event::<ActionEvent<ItemAction<Wield>>>()),
                single_action
                    .pipe(perform_unwield)
                    .pipe(proces_impact)
                    .run_if(on_event::<ActionEvent<ItemAction<Unwield>>>()),
                single_action
                    .pipe(perform_pickup)
                    .pipe(proces_impact)
                    .run_if(on_event::<ActionEvent<ItemAction<Pickup>>>()),
                single_action
                    .pipe(perform_move_item)
                    .pipe(proces_impact)
                    .run_if(on_event::<ActionEvent<ItemAction<MoveItem>>>()),
                single_action
                    .pipe(perform_examine_item)
                    .run_if(on_event::<ActionEvent<ItemAction<ExamineItem>>>()),
                single_action
                    .pipe(perform_change_pace)
                    .run_if(on_event::<ActionEvent<ChangePace>>()),
            ),
            apply_deferred,
            (
                (
                    // actor events
                    update_damaged_characters.run_if(on_event::<ActionEvent<Damage>>()),
                    apply_deferred, // Make sure killed actors are handled early
                    (
                        update_stamina.run_if(on_event::<ActionEvent<StaminaImpact>>()),
                        update_healed_characters.run_if(on_event::<ActionEvent<Healing>>()),
                        update_corpses,
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
                    update_damaged_terrain.run_if(on_event::<TerrainEvent<Damage>>()),
                    apply_deferred, // Make sure destoyed items are handled early
                    toggle_doors.run_if(on_event::<TerrainEvent<Toggle>>()),
                )
                    .chain(),
            ),
            apply_deferred,
            check_items,
        )
            .chain(),
    );
}

pub(crate) fn create_display_behavior_schedule(app: &mut App) {
    app.init_schedule(DisplayBehaviorSchedule);

    app.add_systems(
        DisplayBehaviorSchedule,
        (
            update_transforms,
            update_hidden_item_visibility,
            update_cursor_visibility_on_player_change,
            update_visualization_on_item_move,
            update_visualization_on_focus_move,
            update_visualization_on_weather_change
                .run_if(resource_exists_and_changed::<Timeouts>()),
            update_camera_base.run_if(resource_exists_and_changed::<PlayerActionState>()),
            // sidebar components, in order:
            // (fps is handled elsewhere)
            update_status_time.run_if(resource_exists_and_changed::<Timeouts>()),
            update_status_health,
            update_status_stamina,
            update_status_speed,
            update_status_player_action_state
                .run_if(resource_exists_and_changed::<PlayerActionState>()),
            update_status_player_wielded.run_if(resource_exists_and_changed::<Timeouts>()),
            update_status_enemies.run_if(resource_exists_and_changed::<Timeouts>()),
            update_status_detais.run_if(resource_exists_and_changed::<PlayerActionState>()),
            update_log.run_if(on_event::<Message>()),
        ),
    );
}

pub(crate) fn run_behavior_schedule(world: &mut World) -> DisplayHandling {
    let start = Instant::now();

    let mut count = 0;
    while !waiting_for_user_input(world) && !over_time(&start, count) {
        world.run_schedule(BehaviorSchedule);
        count += 1;
    }

    log_if_slow("run_behavior_schedule", start);

    DisplayHandling::from(count)
}

pub(crate) fn run_behavior_display_schedule(
    In(display_handling): In<DisplayHandling>,
    world: &mut World,
) {
    let start = Instant::now();

    if display_handling == DisplayHandling::Refresh {
        world.run_schedule(DisplayBehaviorSchedule);
    }

    log_if_slow("run_behavior_display_schedule", start);
}

/** All NPC mave a timeout and the player has an empty instruction queue */
fn waiting_for_user_input(world: &mut World) -> bool {
    let mut system_state = SystemState::<(Res<InstructionQueue>,)>::new(world);
    let (instruction_queue,) = system_state.get(world);
    instruction_queue.is_waiting()
}

fn over_time(start: &Instant, count: usize) -> bool {
    let over_time = Duration::from_secs_f32(0.0008) < start.elapsed();
    if over_time {
        eprintln!("run_behavior_schedule could ony handle {count} iterations before the timeout");
    }
    over_time
}
