use crate::application::ApplicationState;
use crate::gameplay::{
    Accessible, Appearance, BaseSpeed, CurrentlyVisible, CurrentlyVisibleBuilder,
    ElevationVisibility, Explored, Focus, GameplaySession, LastSeen, MapAsset, MapMemoryAsset,
    OvermapAsset, OvermapBufferAsset, Player, Pos, SubzoneLevel, ZoneLevel,
};
use crate::loading::LoadingState;
use crate::util::log_if_slow;
use bevy::asset::{AssetLoadError, UntypedAssetLoadFailedEvent};
use bevy::prelude::{
    Assets, Camera, Changed, Children, Commands, EventReader, Font, GlobalTransform, Local, Mesh,
    NextState, ParallelCommands, Parent, Query, Res, ResMut, StandardMaterial, State, Visibility,
    With, Without,
};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[cfg(feature = "log_archetypes")]
use bevy::utils::HashMap;

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
    commands: &mut Commands,
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
    if currently_visible.nearby(SubzoneLevel::from(pos)) {
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
                update_material(commands, children, child_items, last_seen);
            }

            *visibility =
                calculate_visibility(focus, player, pos, elevation_visibility, last_seen, speed);
        }
    }
}

/// Visible to the camera
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

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn update_visibility(
    focus: Focus,
    elevation_visibility: Res<ElevationVisibility>,
    mut session: GameplaySession,
    mut previous_camera_global_transform: Local<GlobalTransform>,
    mut last_elevation_visibility: Local<ElevationVisibility>,
    mut items: Query<(
        Option<&Player>,
        &Pos,
        &mut Visibility,
        &mut LastSeen,
        Option<&BaseSpeed>,
    )>,
    cameras: Query<&GlobalTransform, With<Camera>>,
) {
    let start = Instant::now();

    if session.is_changed() {
        *previous_camera_global_transform = GlobalTransform::default();
        *last_elevation_visibility = ElevationVisibility::default();
    }

    let &camera_global_transform = cameras.single();
    if focus.is_changed()
        || camera_global_transform != *previous_camera_global_transform
        || *elevation_visibility != *last_elevation_visibility
    {
        for (player, &pos, mut visibility, last_seen, speed) in &mut items {
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

        *previous_camera_global_transform = camera_global_transform;
        *last_elevation_visibility = *elevation_visibility;
    }

    log_if_slow("update_visibility", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn update_visualization_on_item_move(
    par_commands: ParallelCommands,
    focus: Focus,
    currently_visible_builder: CurrentlyVisibleBuilder,
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
) {
    let start = Instant::now();

    if moved_items.iter().peekable().peek().is_some() {
        let currently_visible = currently_visible_builder.for_player(true);
        let explored = Arc::new(Mutex::new(&mut *explored));

        for (&pos, mut visibility, mut last_seen, accessible, speed, children) in &mut moved_items {
            par_commands.command_scope(|mut commands| {
                update_visualization(
                    &mut commands,
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
            });
        }
    }

    log_if_slow("update_visualization_on_item_move", start);
}

pub(crate) fn count_assets(
    font_assets: Option<Res<Assets<Font>>>,
    map_assets: Option<Res<Assets<MapAsset>>>,
    map_memory_assets: Option<Res<Assets<MapMemoryAsset>>>,
    overmap_assets: Option<Res<Assets<OvermapAsset>>>,
    overmap_buffer_assets: Option<Res<Assets<OvermapBufferAsset>>>,
    materials: Option<Res<Assets<StandardMaterial>>>,
    meshes: Option<Res<Assets<Mesh>>>,
    mut last_counts: Local<Vec<usize>>,
) {
    if !cfg!(debug_assertions) {
        return;
    }

    let start = Instant::now();

    let counts = vec![
        font_assets.map_or(0, |a| a.len()),
        map_assets.map_or(0, |a| a.len()),
        map_memory_assets.map_or(0, |a| a.len()),
        overmap_assets.map_or(0, |a| a.len()),
        overmap_buffer_assets.map_or(0, |a| a.len()),
        materials.map_or(0, |a| a.len()),
        meshes.map_or(0, |a| a.len()),
    ];

    if *last_counts != counts && counts.iter().any(|c| 0 < *c) {
        println!("{} font assets", counts[0]);
        println!("{} map assets", counts[1]);
        println!("{} map memory assets", counts[2]);
        println!("{} overmap assets", counts[3]);
        println!("{} overmap buffer assets", counts[4]);
        println!("{} material assets", counts[5]);
        println!("{} mesh assets", counts[6]);

        *last_counts = counts;
    }

    log_if_slow("count_assets", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn count_pos(
    zone_levels: Query<(), With<ZoneLevel>>,
    subzone_levels: Query<(), With<SubzoneLevel>>,
    pos: Query<(), With<Pos>>,
    mut last_counts: Local<[usize; 3]>,
) {
    if !cfg!(debug_assertions) {
        return;
    }

    let start = Instant::now();

    let zone_levels = zone_levels.iter().len();
    let subzone_levels = subzone_levels.iter().len();
    let pos = pos.iter().len();

    let counts = [zone_levels, subzone_levels, pos];

    if *last_counts != counts && counts.iter().any(|c| 0 < *c) {
        println!("{subzone_levels} zone levels, {zone_levels} subzone levels, and {pos} positions");

        *last_counts = counts;
    }

    log_if_slow("count_pos", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn check_failed_asset_loading(
    mut fails: EventReader<UntypedAssetLoadFailedEvent>,
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    loading_state: Option<Res<State<LoadingState>>>,
) {
    let start = Instant::now();

    for fail in fails.read() {
        eprintln!("Failed to load asset {}: {:#?}", fail.path, &fail.error);

        match &fail.error {
            AssetLoadError::AssetLoaderError(_) => {
                if loading_state.is_some() {
                    next_application_state.set(ApplicationState::MainMenu);
                }
            }
            _ => {
                assert!(!cfg!(debug_assertions), "Failed to load asset: {fail:#?}");
            }
        }
    }

    log_if_slow("check_failed_asset_loading", start);
}

#[cfg(feature = "log_archetypes")]
#[expect(clippy::needless_pass_by_value)]
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
