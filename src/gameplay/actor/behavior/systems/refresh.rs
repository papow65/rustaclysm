//! These systems run once after [`loop_behavior`].
use crate::prelude::*;
use bevy::prelude::*;
use std::{
    cell::OnceCell,
    sync::{Arc, Mutex},
    time::Instant,
};

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn update_transforms(
    mut obstacles: Query<(&Pos, &mut Transform), Changed<Pos>>,
) {
    let start = Instant::now();

    for (&pos, mut transform) in &mut obstacles {
        transform.translation = pos.vec3();
    }

    log_if_slow("update_transforms", start);
}

/// Independent of `update_transforms`, because the systems affect different entities.
#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn update_peeking_transforms(
    player_action_state: Res<State<PlayerActionState>>,
    players: Query<&Children, With<Player>>,
    mut mesh_transforms: Query<&mut Transform, With<Handle<Mesh>>>,
    initial_offset: Local<OnceCell<Vec3>>,
) {
    let start = Instant::now();

    let state_offset = if let PlayerActionState::Peeking {
        active_target: Some(target),
    } = **player_action_state
    {
        Pos::ORIGIN.horizontal_nbor(target.into()).vec3() * 0.45
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

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn update_hidden_item_visibility(
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
#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn update_visualization_on_player_move(
    par_commands: ParallelCommands,
    focus: Focus,
    currently_visible_builder: CurrentlyVisibleBuilder,
    mut explored: ResMut<Explored>,
    elevation_visibility: Res<ElevationVisibility>,
    mut visualization_update: ResMut<VisualizationUpdate>,
    mut session: GameplaySession,
    mut previous_camera_global_transform: Local<GlobalTransform>,
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

    if session.is_changed() {
        *previous_camera_global_transform = GlobalTransform::default();
    }

    let &camera_global_transform = cameras.single();
    if focus.is_changed()
        || camera_global_transform != *previous_camera_global_transform
        || *visualization_update == VisualizationUpdate::Forced
    {
        let currently_visible = thread_local::ThreadLocal::new();
        let explored = Arc::new(Mutex::new(&mut *explored));
        let only_nearby = *visualization_update != VisualizationUpdate::Forced;

        items.par_iter_mut().for_each(
            |(player, &pos, mut visibility, mut last_seen, accessible, speed, children)| {
                let currently_visible =
                    currently_visible.get_or(|| currently_visible_builder.for_player(only_nearby));

                par_commands.command_scope(|mut commands| {
                    update_visualization(
                        &mut commands,
                        &explored.clone(),
                        currently_visible,
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

        *previous_camera_global_transform = camera_global_transform;
        *visualization_update = VisualizationUpdate::Smart;
    }

    log_if_slow("update_visualization_on_player_move", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn update_visualization_on_weather_change(
    clock: Clock,
    player_action_state: Res<State<PlayerActionState>>,
    mut visualization_update: ResMut<VisualizationUpdate>,
    mut session: GameplaySession,
    mut last_viewing_disttance: Local<Option<usize>>,
    players: Query<&Pos, With<Player>>,
) {
    let start = Instant::now();

    if session.is_changed() {
        *last_viewing_disttance = None;
    }

    let viewing_distance = CurrentlyVisible::viewing_distance(
        &clock,
        Some(&*player_action_state),
        players.single().level,
    );
    if *last_viewing_disttance != viewing_distance {
        *last_viewing_disttance = viewing_distance;

        // Handled by update_visualization_on_player_move next frame
        *visualization_update = VisualizationUpdate::Forced;
    }

    log_if_slow("update_visualization_on_weather_change", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn trigger_refresh(mut visualization_update: ResMut<VisualizationUpdate>) {
    *visualization_update = VisualizationUpdate::Forced;
}

#[cfg(debug_assertions)]
#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn check_items(
    item_parents: Query<Option<&Parent>, Or<(With<Amount>, With<Containable>)>>,
) {
    assert!(
        item_parents.iter().all(|o| o.is_some()),
        "All items should have a parent"
    );
}
