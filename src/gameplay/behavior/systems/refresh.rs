//! These systems run at most once at the end of [`loop_behavior_and_refresh`](`crate::gameplay::behavior::systems::loop::loop_behavior_and_refresh`).

use crate::gameplay::systems::{update_visualization, update_visualization_on_item_move};
use crate::gameplay::{
    Accessible, Appearance, BaseSpeed, Clock, CurrentlyVisible, CurrentlyVisibleBuilder,
    ElevationVisibility, Explored, Focus, GameplayLocal, LastSeen, Player, PlayerActionState, Pos,
    Vehicle, VisualizationUpdate,
};
use crate::util::log_if_slow;
use bevy::ecs::schedule::SystemConfigs;
use bevy::prelude::{
    resource_exists_and_changed, Camera, Changed, Children, GlobalTransform,
    IntoSystemConfigs as _, Local, Mesh3d, ParallelCommands, Parent, Query, RemovedComponents, Res,
    ResMut, State, Transform, Vec3, Visibility, With, Without,
};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::{cell::OnceCell, time::Instant};
use thread_local::ThreadLocal;

pub(super) fn refresh_all() -> SystemConfigs {
    (
        update_transforms,
        update_peeking_transforms.run_if(resource_exists_and_changed::<State<PlayerActionState>>),
        update_hidden_item_visibility,
        update_visualization_on_item_move,
        update_visualization_on_player_move,
        update_visualization_on_weather_change,
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
    players: Query<&Children, With<Player>>,
    mut mesh_transforms: Query<&mut Transform, With<Mesh3d>>,
    initial_offset: Local<OnceCell<Vec3>>,
) {
    let start = Instant::now();

    let state_offset = if let PlayerActionState::Peeking { direction } = **player_action_state {
        Pos::ORIGIN.horizontal_nbor(direction.into()).vec3() * 0.45
    } else {
        Vec3::ZERO
    };

    let children = players.single();
    for child in children {
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
    mut explored: ResMut<Explored>,
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
    child_items: Query<&Appearance, (With<Parent>, Without<Pos>)>,
    cameras: Query<&GlobalTransform, With<Camera>>,
) {
    let start = Instant::now();

    let &camera_global_transform = cameras.single();
    let camera_moved = camera_global_transform != *previous_camera_global_transform.get();

    if focus.is_changed() || camera_moved || visualization_update.forced() {
        let currently_visible = ThreadLocal::new();
        let explored = Arc::new(Mutex::new(&mut *explored));

        items.par_iter_mut().for_each(
            |(player, &pos, mut visibility, mut last_seen, accessible, speed, children)| {
                let currently_visible = currently_visible.get_or(|| {
                    RefCell::new(currently_visible_builder.for_player(!visualization_update.forced()))
                });

                par_commands.command_scope(|mut commands| {
                    update_visualization(
                        &mut commands,
                        &explored.clone(),
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
                    );
                });
            },
        );

        println!("{}x visualization updated", items.iter().len());

        *previous_camera_global_transform.get() = camera_global_transform;
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
    players: Query<&Pos, With<Player>>,
) {
    let start = Instant::now();

    let viewing_distance = CurrentlyVisible::viewing_distance(
        &clock,
        Some(&*player_action_state),
        players.single().level,
    );
    if *last_viewing_disttance.get() != viewing_distance {
        *last_viewing_disttance.get() = viewing_distance;

        // Handled by update_visualization_on_player_move next frame
        *visualization_update = VisualizationUpdate::Forced;
    }

    log_if_slow("update_visualization_on_weather_change", start);
}
