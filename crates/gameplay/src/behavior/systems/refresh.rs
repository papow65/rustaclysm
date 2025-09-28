//! These systems run at most once at the end of [`loop_behavior_and_refresh`](`crate::behavior::systems::loop::loop_behavior_and_refresh`).

use crate::systems::{update_visualization, update_visualization_on_item_move};
use crate::{
    Accessible, BaseSpeed, Clock, CurrentlyVisible, CurrentlyVisibleBuilder, ElevationVisibility,
    Focus, Player, PlayerActionState, Vehicle, VisualizationUpdate,
};
use bevy::ecs::schedule::{IntoScheduleConfigs as _, ScheduleConfigs};
use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::{
    Camera, Changed, ChildOf, Children, GlobalTransform, Local, Mesh3d, MessageWriter,
    ParallelCommands, Query, RemovedComponents, Res, ResMut, Single, State, Transform, Vec3,
    Visibility, With, Without, debug, resource_exists_and_changed,
};
use gameplay_cdda::Exploration;
use gameplay_local::GameplayLocal;
use gameplay_location::Pos;
use gameplay_model::{Appearance, LastSeen};
use std::cell::RefCell;
use std::{cell::OnceCell, time::Instant};
use thread_local::ThreadLocal;
use util::log_if_slow;

pub(super) fn refresh_all() -> ScheduleConfigs<ScheduleSystem> {
    (
        update_transforms,
        update_peeking_transforms.run_if(resource_exists_and_changed::<State<PlayerActionState>>),
        update_hidden_item_visibility,
        ((
            update_visualization_on_item_move,
            (
                update_visualization_on_weather_change,
                update_visualization_on_player_move,
            )
                .chain(),
        ),)
            .chain(),
    )
        .into_configs()
}

fn update_transforms(
    mut obstacles: Query<(&Pos, &mut Transform), (Changed<Pos>, Without<Vehicle>)>,
) {
    let start = Instant::now();

    for (&pos, mut transform) in &mut obstacles {
        transform.translation = pos.vec3();
    }

    log_if_slow("update_transforms", start);
}

/// Independent of `update_transforms`, because the systems affect different entities.
#[expect(clippy::needless_pass_by_value)]
fn update_peeking_transforms(
    player_action_state: Res<State<PlayerActionState>>,
    player_children: Single<&Children, With<Player>>,
    mut mesh_transforms: Query<&mut Transform, With<Mesh3d>>,
    initial_offset: Local<OnceCell<Vec3>>,
) {
    let start = Instant::now();

    let state_offset = if let PlayerActionState::Peeking { direction } = **player_action_state {
        Pos::ORIGIN.horizontal_nbor(direction.into()).vec3() * 0.45
    } else {
        Vec3::ZERO
    };

    for child in *player_children {
        if let Ok(mut transform) = mesh_transforms.get_mut(*child) {
            transform.translation =
                state_offset + *initial_offset.get_or_init(|| transform.translation);
        }
    }

    log_if_slow("update_peeking_transforms", start);
}

fn update_hidden_item_visibility(
    mut hidden_items: Query<&mut Visibility, Without<Pos>>,
    mut removed_positions: RemovedComponents<Pos>,
) {
    let start = Instant::now();

    // TODO use update_visualization
    for entity in removed_positions.read() {
        if let Ok(mut visibility) = hidden_items.get_mut(entity) {
            *visibility = Visibility::Hidden;
        }
    }

    log_if_slow("update_visibility_for_hidden_items", start);
}

/// This is a slow system. For performance, it is only ran once after [`BehaviorSchedule::run`[, instead of after every action
#[expect(clippy::needless_pass_by_value)]
fn update_visualization_on_player_move(
    par_commands: ParallelCommands,
    focus: Focus,
    currently_visible_builder: CurrentlyVisibleBuilder,
    mut explorations: MessageWriter<Exploration>,
    elevation_visibility: Res<ElevationVisibility>,
    mut visualization_update: ResMut<VisualizationUpdate>,
    mut previous_camera_global_transform: GameplayLocal<GlobalTransform>,
    mut items: Query<(
        Option<&Player>,
        &Pos,
        &mut Visibility,
        &mut LastSeen,
        Option<&Accessible>,
        Option<&BaseSpeed>,
        &Children,
    )>,
    child_items: Query<&Appearance, (With<ChildOf>, Without<Pos>)>,
    camera_global_transform: Single<&GlobalTransform, With<Camera>>,
) {
    let start = Instant::now();

    let camera_moved = **camera_global_transform != *previous_camera_global_transform.get();

    if focus.is_changed() || camera_moved || visualization_update.forced() {
        let currently_visible = ThreadLocal::new();
        let new_explorations = ThreadLocal::new();

        items.par_iter_mut().for_each(
            |(player, &pos, mut visibility, mut last_seen, accessible, speed, children)| {
                let currently_visible = currently_visible.get_or(|| {
                    RefCell::new(
                        currently_visible_builder.for_player(!visualization_update.forced()),
                    )
                });
                let new_explorations =
                    new_explorations.get_or(RefCell::<Vec<Exploration>>::default);

                let exploration = par_commands.command_scope(|mut commands| {
                    update_visualization(
                        &mut commands,
                        &mut currently_visible.borrow_mut(),
                        *elevation_visibility,
                        &focus,
                        player,
                        pos,
                        &mut visibility,
                        &mut last_seen,
                        accessible,
                        speed,
                        children,
                        &child_items,
                    )
                });

                if let Some(exploration) = exploration {
                    new_explorations.borrow_mut().push(exploration);
                }
            },
        );

        for ref_cell in new_explorations {
            explorations.write_batch(ref_cell.into_inner());
        }

        debug!("{}x visualization updated", items.iter().len());

        *previous_camera_global_transform.get() = **camera_global_transform;
        (*visualization_update).reset();
    }

    log_if_slow("update_visualization_on_player_move", start);
}

#[expect(clippy::needless_pass_by_value)]
fn update_visualization_on_weather_change(
    clock: Clock,
    player_action_state: Res<State<PlayerActionState>>,
    mut visualization_update: ResMut<VisualizationUpdate>,
    mut last_viewing_disttance: GameplayLocal<Option<u8>>,
    player_pos: Single<&Pos, With<Player>>,
) {
    let start = Instant::now();

    let viewing_distance =
        CurrentlyVisible::viewing_distance(&clock, Some(&*player_action_state), player_pos.level);
    if *last_viewing_disttance.get() != viewing_distance {
        *last_viewing_disttance.get() = viewing_distance;

        // Handled by update_visualization_on_player_move next frame
        *visualization_update = VisualizationUpdate::Forced;
    }

    log_if_slow("update_visualization_on_weather_change", start);
}
