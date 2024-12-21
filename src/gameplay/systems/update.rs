use crate::gameplay::events::Exploration;
use crate::gameplay::{
    Accessible, Appearance, BaseSpeed, CurrentlyVisible, CurrentlyVisibleBuilder,
    ElevationVisibility, Focus, GameplayLocal, LastSeen, Player, Pos, SubzoneLevel,
};
use crate::util::log_if_slow;
use bevy::prelude::{
    Camera, Changed, Children, Commands, EventWriter, GlobalTransform, ParallelCommands, Parent,
    Query, Res, Visibility, With, Without,
};
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
    currently_visible: &mut CurrentlyVisible,
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
) -> Option<Exploration> {
    let mut exploration = None;

    if currently_visible.nearby(SubzoneLevel::from(pos)) {
        let previously_seen = last_seen.clone();

        let visible = currently_visible.can_see(pos, accessible);
        // TODO check if there is enough light
        last_seen.update(&visible);

        if last_seen != &LastSeen::Never {
            if last_seen != &previously_seen {
                if previously_seen == LastSeen::Never {
                    exploration = Some(Exploration::Pos(pos));
                }

                // TODO select an appearance based on amount of perceived light
                update_material(commands, children, child_items, last_seen);
            }

            *visibility =
                calculate_visibility(focus, player, pos, elevation_visibility, last_seen, speed);
        }
    }

    exploration
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
    mut previous_camera_global_transform: GameplayLocal<GlobalTransform>,
    mut last_elevation_visibility: GameplayLocal<ElevationVisibility>,
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

    let &camera_global_transform = cameras.single();
    if focus.is_changed()
        || camera_global_transform != *previous_camera_global_transform.get()
        || *elevation_visibility != *last_elevation_visibility.get()
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

        *previous_camera_global_transform.get() = camera_global_transform;
        *last_elevation_visibility.get() = *elevation_visibility;
    }

    log_if_slow("update_visibility", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn update_visualization_on_item_move(
    par_commands: ParallelCommands,
    mut explorations: EventWriter<Exploration>,
    focus: Focus,
    currently_visible_builder: CurrentlyVisibleBuilder,
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
        let mut currently_visible = currently_visible_builder.for_player(true);

        for (&pos, mut visibility, mut last_seen, accessible, speed, children) in &mut moved_items {
            let exploration = par_commands.command_scope(|mut commands| {
                update_visualization(
                    &mut commands,
                    &mut currently_visible,
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
                )
            });

            if let Some(exploration) = exploration {
                explorations.send(exploration);
            }
        }
    }

    log_if_slow("update_visualization_on_item_move", start);
}
