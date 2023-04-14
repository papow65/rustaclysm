use crate::prelude::*;
use bevy::prelude::*;

const FONT_SIZE: f32 = 40.0;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_loading(mut commands: Commands, fonts: Res<Fonts>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                size: Size::all(Val::Percent(100.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .insert(LoadingRoot)
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(250.0), Val::Px(70.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: DEFAULT_BUTTON_COLOR.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Loading...",
                        TextStyle {
                            font: fonts.default(),
                            font_size: FONT_SIZE,
                            color: DEFAULT_TEXT_COLOR,
                        },
                    ));
                });
        });
}

/** We start loading after the frame that spawns the loading indicator, to ensure the loading indicator is visible to the user. */
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn start_gameplay(
    application_state: Res<State<ApplicationState>>,
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    mut delay: Local<bool>,
) {
    if *delay && application_state.0 != ApplicationState::Gameplay {
        next_application_state.set(ApplicationState::Gameplay);
    } else {
        *delay = true;
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn finish_loading(
    mut next_progress_state: ResMut<NextState<ProgressScreenState>>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    explored: Res<Explored>,
    subzone_level_entities: Res<SubzoneLevelEntities>,
    maps: Res<Maps>,
    mut counter: Local<u8>,
) {
    if 3 < *counter {
        println!(
            "Loading {} {:?} {:?} {:?}",
            *counter,
            explored.loaded(),
            subzone_level_entities.loaded(),
            maps.loading.is_empty()
        );

        // subzone_level_entities sometimes fails to load for unknown reason. In that case, we give control back to the user after a delay.
        let subzones_loaded = subzone_level_entities.loaded() || *counter == u8::MAX;

        if subzones_loaded && explored.loaded() && maps.loading.is_empty() {
            eprintln!("Loading complete");
            next_progress_state.set(ProgressScreenState::Complete);
            next_gameplay_state.set(GameplayScreenState::Base);
        }
    }

    *counter = counter.saturating_add(1);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn despawn_loading(
    mut commands: Commands,
    root_entities: Query<Entity, With<LoadingRoot>>,
) {
    if let Ok(root_entity) = root_entities.get_single() {
        commands.entity(root_entity).despawn_recursive();
    }
}
