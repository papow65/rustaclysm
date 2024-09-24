use crate::application::ApplicationState;
use crate::common::{Fonts, DEFAULT_BUTTON_COLOR, DEFAULT_TEXT_COLOR};
use crate::gameplay::{Explored, GameplayScreenState, SubzoneLevelEntities};
use crate::loading::ProgressScreenState;
use bevy::prelude::{
    AlignItems, Assets, BuildChildren, Commands, JustifyContent, Local, NextState, NodeBundle,
    PositionType, Res, ResMut, State, StateScoped, Style, TextBundle, Val, ZIndex,
};
use cdda_json_files::{Map, Overmap, OvermapBuffer};

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn spawn_loading(mut commands: Commands, fonts: Res<Fonts>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Style::default()
                },
                z_index: ZIndex::Global(3),
                ..NodeBundle::default()
            },
            StateScoped(ProgressScreenState::Loading),
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(250.0),
                        height: Val::Px(70.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Style::default()
                    },
                    background_color: DEFAULT_BUTTON_COLOR.into(),
                    ..NodeBundle::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Loading...",
                        fonts.large(DEFAULT_TEXT_COLOR),
                    ));
                });
        });
}

/// We start loading after the frame that spawns the loading indicator, to ensure the loading indicator is visible to the user.
#[expect(clippy::needless_pass_by_value)]
pub(crate) fn start_gameplay(
    application_state: Res<State<ApplicationState>>,
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    mut delay: Local<bool>,
) {
    if *delay && application_state.get() != &ApplicationState::Gameplay {
        next_application_state.set(ApplicationState::Gameplay);
    } else {
        *delay = true;
    }
}

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn finish_loading(
    mut next_progress_state: ResMut<NextState<ProgressScreenState>>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    overmap_assets: Res<Assets<Overmap>>,
    overmap_buffer_assets: Res<Assets<OvermapBuffer>>,
    map_assets: Res<Assets<Map>>,
    explored: Res<Explored>,
    subzone_level_entities: Res<SubzoneLevelEntities>,
    mut counter: Local<u8>,
) {
    if 3 < *counter {
        println!(
            "Loading status {}: {}, {}, {}, {:?}, and {:?}",
            *counter,
            overmap_assets.len(),
            overmap_buffer_assets.len(),
            map_assets.len(),
            explored.loaded(),
            subzone_level_entities.loaded(),
        );

        // subzone_level_entities sometimes fails to load for unknown reason. In that case, we give control back to the user after a delay.
        let subzones_loaded = subzone_level_entities.loaded() || *counter == u8::MAX;

        if subzones_loaded
            && 0 < overmap_assets.len()
            && 0 < overmap_buffer_assets.len()
            && 0 < map_assets.len()
            && explored.loaded()
        {
            eprintln!("Loading complete");
            next_progress_state.set(ProgressScreenState::Complete);
            next_gameplay_state.set(GameplayScreenState::Base);
        }
    }

    *counter = counter.saturating_add(1);
}
