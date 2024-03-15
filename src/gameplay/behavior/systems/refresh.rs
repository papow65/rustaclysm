use crate::prelude::*;
use bevy::prelude::*;
use std::{
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

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn update_visualization_on_player_move(
    commands: Commands,
    focus_state: Res<State<FocusState>>,
    player_action_state: Res<PlayerActionState>,
    envir: Envir,
    clock: Clock,
    mut explored: ResMut<Explored>,
    elevation_visibility: Res<ElevationVisibility>,
    mut visualization_update: ResMut<VisualizationUpdate>,
    mut session: GameplaySession,
    mut last_focus: Local<Focus>,
    mut previous_camera_global_transform: Local<GlobalTransform>,
    mut last_elevation_visibility: Local<ElevationVisibility>,
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
    players: Query<&Pos, With<Player>>,
    cameras: Query<&GlobalTransform, With<Camera>>,
) {
    let start = Instant::now();

    if let (
        &FocusState::ExaminingPos(_) | &FocusState::ExaminingZoneLevel(_),
        VisualizationUpdate::Smart,
    ) = (&**focus_state, *visualization_update)
    {
        update_visibility(
            focus_state,
            elevation_visibility,
            session,
            last_focus,
            previous_camera_global_transform,
            last_elevation_visibility,
            items,
            players,
            cameras,
        );
    } else {
        if session.is_changed() {
            *last_focus = Focus::default();
            *previous_camera_global_transform = GlobalTransform::default();
            *last_elevation_visibility = ElevationVisibility::default();
        }

        let &player_pos = players.single();
        let focus = Focus::new(&focus_state, player_pos);
        let &camera_global_transform = cameras.single();
        if focus != *last_focus
            || camera_global_transform != *previous_camera_global_transform
            || *elevation_visibility != *last_elevation_visibility
            || *visualization_update == VisualizationUpdate::Forced
        {
            let currently_visible = thread_local::ThreadLocal::new();
            let commands = Arc::new(Mutex::new(commands));
            let explored = Arc::new(Mutex::new(&mut *explored));

            items.par_iter_mut().for_each(
                |(player, &pos, mut visibility, mut last_seen, accessible, speed, children)| {
                    let currently_visible = currently_visible.get_or(|| {
                        envir.currently_visible(&clock, &player_action_state, player_pos)
                    });

                    update_visualization(
                        &commands.clone(),
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
                },
            );

            println!("{}x visualization updated", items.iter().len());

            *last_focus = focus;
            *previous_camera_global_transform = camera_global_transform;
            *last_elevation_visibility = *elevation_visibility;
            *visualization_update = VisualizationUpdate::Smart;
        }
    }

    log_if_slow("update_visualization_on_focus_move", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn update_visualization_on_weather_change(
    clock: Clock,
    player_action_state: Res<PlayerActionState>,
    mut visualization_update: ResMut<VisualizationUpdate>,
    mut session: GameplaySession,
    mut last_viewing_disttance: Local<Option<usize>>,
    players: Query<&Pos, With<Player>>,
) {
    let start = Instant::now();

    if session.is_changed() {
        *last_viewing_disttance = None;
    }

    let player_pos = players.single();
    let viewing_distance =
        CurrentlyVisible::viewing_distance(&clock, &player_action_state, player_pos.level);
    if *last_viewing_disttance != viewing_distance {
        *last_viewing_disttance = viewing_distance;

        // Handled by update_visualization_on_focus_move next frame
        *visualization_update = VisualizationUpdate::Forced;
    }

    log_if_slow("update_visualization_on_weather_change", start);
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
