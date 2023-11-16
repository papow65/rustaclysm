use crate::prelude::*;
use bevy::prelude::*;
use std::time::Instant;

#[cfg(feature = "log_archetypes")]
use bevy::utils::HashMap;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_transforms(mut obstacles: Query<(&Pos, &mut Transform), Changed<Pos>>) {
    let start = Instant::now();

    for (&pos, mut transform) in &mut obstacles {
        transform.translation = pos.vec3();
    }

    log_if_slow("update_transforms", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_hidden_item_visibility(
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
pub(crate) fn update_cursor_visibility_on_player_change(
    player_action_state: Res<PlayerActionState>,
    mut curors: Query<(&mut Visibility, &mut Transform), With<ExamineCursor>>,
) {
    let start = Instant::now();

    if let Ok((mut visibility, mut transform)) = curors.get_single_mut() {
        let examine_pos = matches!(*player_action_state, PlayerActionState::ExaminingPos(_));
        let examine_zone_level = matches!(
            *player_action_state,
            PlayerActionState::ExaminingZoneLevel(_)
        );
        *visibility = if examine_pos || examine_zone_level {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        transform.scale = if examine_zone_level {
            Vec3::splat(24.0)
        } else {
            Vec3::ONE
        };
    }

    log_if_slow("update_cursor_visibility_on_player_change", start);
}

fn update_material(
    commands: &mut Commands,
    children: &Children,
    child_items: &Query<&Appearance, (With<Parent>, Without<Pos>)>,
    last_seen: &LastSeen,
) {
    for &child in children {
        if let Ok(child_appearance) = child_items.get(child) {
            commands
                .entity(child)
                .insert(child_appearance.material(last_seen));
        }
    }
}

fn update_visualization(
    commands: &mut Commands,
    explored: &mut Explored,
    currently_visible: &CurrentlyVisible,
    elevation_visibility: ElevationVisibility,
    focus: &Focus,
    player: Option<&Player>,
    pos: Pos,
    visibility: &mut Visibility,
    last_seen: &mut LastSeen,
    accessible: Option<&Accessible>,
    speed: Option<&BaseSpeed>,
    children: &Children,
    child_items: &Query<&Appearance, (With<Parent>, Without<Pos>)>,
) {
    let previously_seen = last_seen.clone();

    let visible = currently_visible.can_see(pos, accessible);
    // TODO check if there is enough light
    last_seen.update(&visible);

    if last_seen != &LastSeen::Never {
        if last_seen != &previously_seen {
            if previously_seen == LastSeen::Never {
                explored.mark_pos_seen(pos);
            }

            // TODO select an appearance based on amount of perceived light
            update_material(commands, children, child_items, last_seen);
        }

        *visibility =
            calculate_visibility(focus, player, pos, elevation_visibility, last_seen, speed);
    }
}

fn calculate_visibility(
    focus: &Focus,
    player: Option<&Player>,
    pos: Pos,
    elevation_visibility: ElevationVisibility,
    last_seen: &LastSeen,
    speed: Option<&BaseSpeed>,
) -> Visibility {
    // The player character can see things not shown to the player, like the top of a tower when walking next to it.
    if focus.is_pos_shown(pos, elevation_visibility) && last_seen.shown(player, speed) {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_visualization_on_item_move(
    mut commands: Commands,
    player_action_state: Res<PlayerActionState>,
    envir: Envir,
    clock: Clock,
    mut explored: ResMut<Explored>,
    elevation_visibility: Res<ElevationVisibility>,
    mut moved_items: Query<
        (
            &Pos,
            &mut Visibility,
            &mut LastSeen,
            Option<&Accessible>,
            Option<&BaseSpeed>,
            &Children,
        ),
        Changed<Pos>,
    >,
    child_items: Query<&Appearance, (With<Parent>, Without<Pos>)>,
    players: Query<&Pos, With<Player>>,
) {
    let start = Instant::now();

    if moved_items.iter().peekable().peek().is_some() {
        let &player_pos = players.single();
        let focus = Focus::new(&player_action_state, player_pos);
        let currently_visible = envir.currently_visible(&clock, &player_action_state, player_pos);

        for (&pos, mut visibility, mut last_seen, accessible, speed, children) in &mut moved_items {
            update_visualization(
                &mut commands,
                &mut explored,
                &currently_visible,
                *elevation_visibility,
                &focus,
                None,
                pos,
                &mut visibility,
                &mut last_seen,
                accessible,
                speed,
                children,
                &child_items,
            );
        }
    }

    log_if_slow("update_visualization_on_item_move", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_visualization_on_focus_move(
    mut commands: Commands,
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

    if session.is_changed() {
        *last_focus = Focus::default();
        *previous_camera_global_transform = GlobalTransform::default();
        *last_elevation_visibility = ElevationVisibility::default();
    }

    let &player_pos = players.single();
    let focus = Focus::new(&player_action_state, player_pos);
    let &camera_global_transform = cameras.single();
    if focus != *last_focus
        || camera_global_transform != *previous_camera_global_transform
        || *elevation_visibility != *last_elevation_visibility
        || *visualization_update == VisualizationUpdate::Forced
    {
        if let (
            &PlayerActionState::ExaminingPos(_) | &PlayerActionState::ExaminingZoneLevel(_),
            VisualizationUpdate::Smart,
        ) = (&*player_action_state, *visualization_update)
        {
            for (player, &pos, mut visibility, last_seen, _, speed, _) in &mut items {
                if *last_seen != LastSeen::Never {
                    *visibility = calculate_visibility(
                        &focus,
                        player,
                        pos,
                        *elevation_visibility,
                        &last_seen,
                        speed,
                    );
                }
            }

            println!("{}x visibility updated", items.iter().len());
        } else {
            let currently_visible =
                envir.currently_visible(&clock, &player_action_state, player_pos);

            // Using parallel iteration was worse for performance using bevy 0.11
            for (player, &pos, mut visibility, mut last_seen, accessible, speed, children) in
                &mut items
            {
                update_visualization(
                    &mut commands,
                    &mut explored,
                    &currently_visible,
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
            }

            println!("{}x visualization updated", items.iter().len());
        }

        *last_focus = focus;
        *previous_camera_global_transform = camera_global_transform;
        *last_elevation_visibility = *elevation_visibility;
        *visualization_update = VisualizationUpdate::Smart;
    }

    log_if_slow("update_visualization_on_focus_move", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_visualization_on_weather_change(
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

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_camera_base(
    player_state: Res<PlayerActionState>,
    players: Query<&Pos, With<Player>>,
    mut camera_bases: Query<&mut Transform, (With<CameraBase>, Without<Camera3d>)>,
) {
    let start = Instant::now();

    let pos = players.single();

    for mut transform in &mut camera_bases {
        transform.translation = match *player_state {
            PlayerActionState::ExaminingPos(target) => target.vec3() - pos.vec3(),
            PlayerActionState::ExaminingZoneLevel(target) => {
                target.base_pos().vec3() - pos.vec3() + Vec3::new(11.5, 0.0, 11.5)
            }
            _ => Vec3::ZERO,
        };
    }

    log_if_slow("update_camera", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_camera_offset(
    camera_offset: Res<CameraOffset>,
    mut cameras: Query<&mut Transform, With<Camera3d>>,
) {
    let start = Instant::now();

    let mut transform = cameras.single_mut();
    transform.translation = camera_offset.offset();
    transform.look_at(Vec3::ZERO, Vec3::Y);

    log_if_slow("update_camera", start);
}

#[cfg(debug_assertions)]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn check_items(
    item_parents: Query<Option<&Parent>, Or<(With<Amount>, With<Containable>)>>,
) {
    assert!(
        item_parents.iter().all(|o| o.is_some()),
        "All items should have a parent"
    );
}

#[cfg(feature = "log_archetypes")]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn log_archetypes(world: &mut World) {
    let component_names = world
        .components()
        .iter()
        .map(|component| {
            (component.id(), {
                let name = component.name();
                let (base, brackets) = name.split_once('<').unwrap_or((name, ""));
                let short_base = base.rsplit_once(':').unwrap_or(("", base)).1;
                String::from(short_base) + (if brackets.is_empty() { "" } else { "<" }) + brackets
            })
        })
        .collect::<HashMap<_, _>>();

    for archetype in world.archetypes().iter() {
        if !archetype.is_empty() {
            println!(
                "{:?} {:?} {:?}",
                archetype.id(),
                archetype.len(),
                archetype
                    .components()
                    .map(|component| component_names
                        .get(&component)
                        .cloned()
                        .unwrap_or(String::from("[???]")))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
}
