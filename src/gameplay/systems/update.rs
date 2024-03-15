use crate::prelude::*;
use bevy::prelude::*;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

#[cfg(feature = "log_archetypes")]
use bevy::utils::HashMap;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_focus_cursor_visibility(
    focus_state: Res<State<FocusState>>,
    mut curors: Query<(&mut Visibility, &mut Transform), With<ExamineCursor>>,
) {
    let start = Instant::now();

    if let Ok((mut visibility, mut transform)) = curors.get_single_mut() {
        let examine_pos = matches!(**focus_state, FocusState::ExaminingPos(_));
        let examine_zone_level = matches!(**focus_state, FocusState::ExaminingZoneLevel(_));
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

pub(crate) fn update_visualization(
    commands: &Arc<Mutex<Commands>>,
    explored: &Arc<Mutex<&mut Explored>>,
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
                explored.lock().expect("Unpoisoned").mark_pos_seen(pos);
            }

            // TODO select an appearance based on amount of perceived light
            update_material(
                &mut commands.lock().expect("Unpoisoned"),
                children,
                child_items,
                last_seen,
            );
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

/** 'items' is more complex than needed by [`calculate_visibility`], because of compatibility with \
 *[`update_visualization_on_focus_move()`]. */
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_visibility(
    focus_state: Res<State<FocusState>>,
    elevation_visibility: Res<ElevationVisibility>,
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
    let focus = Focus::new(&focus_state, player_pos);
    let &camera_global_transform = cameras.single();
    if focus != *last_focus
        || camera_global_transform != *previous_camera_global_transform
        || *elevation_visibility != *last_elevation_visibility
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

        *last_focus = focus;
        *previous_camera_global_transform = camera_global_transform;
        *last_elevation_visibility = *elevation_visibility;
    }

    log_if_slow("update_visibility", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_visualization_on_item_move(
    commands: Commands,
    focus_state: Res<State<FocusState>>,
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
        let focus = Focus::new(&focus_state, player_pos);
        let currently_visible = envir.currently_visible(&clock, &player_action_state, player_pos);
        let commands = Arc::new(Mutex::new(commands));
        let explored = Arc::new(Mutex::new(&mut *explored));

        for (&pos, mut visibility, mut last_seen, accessible, speed, children) in &mut moved_items {
            update_visualization(
                &commands.clone(),
                &explored.clone(),
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
pub(crate) fn update_camera_base(
    focus_state: Res<State<FocusState>>,
    players: Query<&Pos, With<Player>>,
    mut camera_bases: Query<&mut Transform, (With<CameraBase>, Without<Camera3d>)>,
) {
    let start = Instant::now();

    let player_pos = *players.single();
    camera_bases.single_mut().translation = Focus::new(&focus_state, player_pos).offset(player_pos);

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
pub(crate) fn count_assets(
    map_assets: Option<Res<Assets<Map>>>,
    map_memory_assets: Option<Res<Assets<MapMemory>>>,
    overmap_assets: Option<Res<Assets<Overmap>>>,
    overmap_buffer_assets: Option<Res<Assets<OvermapBuffer>>>,
    materials: Option<Res<Assets<StandardMaterial>>>,
    meshes: Option<Res<Assets<Mesh>>>,
    mut last_counts: Local<Vec<usize>>,
) {
    let start = Instant::now();

    let counts = vec![
        map_assets.map_or(0, |a| a.len()),
        map_memory_assets.map_or(0, |a| a.len()),
        overmap_assets.map_or(0, |a| a.len()),
        overmap_buffer_assets.map_or(0, |a| a.len()),
        materials.map_or(0, |a| a.len()),
        meshes.map_or(0, |a| a.len()),
    ];

    if *last_counts != counts && counts.iter().any(|c| 0 < *c) {
        println!("{} map assets", counts[0]);
        println!("{} map memory assets", counts[1]);
        println!("{} overmap assets", counts[2]);
        println!("{} overmap buffer assets", counts[3]);
        println!("{} material assets", counts[4]);
        println!("{} mesh assets", counts[5]);

        *last_counts = counts;
    }

    log_if_slow("count_assets", start);
}

#[cfg(debug_assertions)]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn count_zones(
    //zones: Query<&Zone>,
    zone_levels: Query<&ZoneLevel>,
    subzone_levels: Query<&SubzoneLevel>,
    mut last_counts: Local<Vec<usize>>,
) {
    let start = Instant::now();

    let counts = vec![
        /*zones.len(),*/ zone_levels.iter().len(),
        subzone_levels.iter().len(),
    ];

    if *last_counts != counts && counts.iter().any(|c| 0 < *c) {
        //println!("{} zones", counts[0]);
        println!("{} zone levels", counts[0]);
        println!("{} subzone levels", counts[1]);

        *last_counts = counts;
    }

    log_if_slow("count_zones", start);
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
