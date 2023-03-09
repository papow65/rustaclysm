use crate::prelude::*;
use bevy::{ecs::system::SystemState, prelude::*};
use std::time::{Duration, Instant};

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum UpdateSet {
    ProcessKeyboard,
    ManageBehavior,
    FlushBehavior,
    ApplyEffects,
    FlushEffects,
}

pub(crate) struct RustaclysmPlugin;

impl Plugin for RustaclysmPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa::default())
            .insert_resource(AmbientLight {
                brightness: 0.2,
                ..AmbientLight::default()
            })
            .insert_resource(Infos::new())
            .insert_resource(Location::default())
            .insert_resource(SubzoneLevelEntities::default())
            .insert_resource(ZoneLevelEntities::default())
            .insert_resource(InstructionQueue::default())
            .insert_resource(RelativeSegments::new())
            .insert_resource(TileCaches::default());

        app.add_schedule(BehaviorSchedule, behavior_schedule());

        // executed once at startup
        app.add_startup_systems(
            (
                create_secondairy_resources,
                apply_system_buffers,
                spawn_initial_entities,
                spawn_hud,
                apply_system_buffers,
                maximize_window,
            )
                .chain(),
        );
        app.add_system(manage_mouse_input.before(update_camera));

        // executed every frame
        app.add_systems((manage_keyboard_input,).in_set(UpdateSet::ProcessKeyboard));
        app.add_systems((run_behavior_schedule,).after(UpdateSet::ProcessKeyboard));
        app.add_systems(
            (
                update_transforms,
                update_hidden_item_visibility,
                update_cursor_visibility_on_player_change,
                update_visualization_on_item_move,
                update_visualization_on_focus_move,
                update_camera,
            )
                .after(UpdateSet::FlushEffects),
        );
        app.add_system(update_log);
        app.add_system(update_status_fps);
        app.add_system(update_status_time);
        app.add_system(update_status_health);
        app.add_system(update_status_speed);
        app.add_system(update_status_player_state);
        app.add_system(update_status_detais);
        app.add_system(spawn_zones_for_camera.after(update_camera));
        app.add_system(update_collapsed_zone_levels.after(update_camera));

        app.add_system(check_delay.in_base_set(CoreSet::Last));
    }
}

fn behavior_schedule() -> Schedule {
    let mut behavior_schedule = Schedule::new();
    behavior_schedule.add_systems((manage_characters,).in_set(UpdateSet::ManageBehavior));
    behavior_schedule.add_systems(
        (apply_system_buffers,)
            .in_set(UpdateSet::FlushBehavior)
            .after(UpdateSet::ManageBehavior),
    );
    behavior_schedule.add_systems(
        (
            manage_game_over,
            toggle_doors,
            update_damaged_characters,
            update_damaged_items,
        )
            .in_set(UpdateSet::ApplyEffects)
            .after(UpdateSet::FlushBehavior),
    );
    behavior_schedule.add_systems(
        (apply_system_buffers,)
            .in_set(UpdateSet::FlushEffects)
            .after(UpdateSet::ApplyEffects),
    );
    behavior_schedule
}

fn run_behavior_schedule(world: &mut World) {
    let start = Instant::now();

    let mut count = 0;
    while !waiting_for_user_input(world) && !over_time(&start, count) {
        let iteration = Instant::now();

        world.run_schedule(BehaviorSchedule);

        count += 1;
        println!(
            "iteration {count} of run_behavior_schedule took {:?} after appling ({:?} since start)",
            iteration.elapsed(),
            start.elapsed(),
        );
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
