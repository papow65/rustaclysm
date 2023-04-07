use crate::prelude::*;
use bevy::prelude::*;

const FONT_SIZE: f32 = 40.0;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    eprintln!("spawn_loading");
    let font = default_font(&asset_server);

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(100.0)),
                flex_direction: FlexDirection::Column,
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
                            font,
                            font_size: FONT_SIZE,
                            color: DEFAULT_TEXT_COLOR,
                        },
                    ));
                });
        });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn finish_loading(
    mut next_state: ResMut<NextState<GameplayScreenState>>,
    explored: Res<Explored>,
    subzone_level_entities: Res<SubzoneLevelEntities>,
    maps: Res<Maps>,
    mut counter: Local<u8>,
) {
    if 3 < *counter {
        eprintln!(
            "Loading {:?} {:?} {:?}",
            explored.loaded(),
            subzone_level_entities.loaded(),
            maps.loading.is_empty()
        );
        if explored.loaded() && subzone_level_entities.loaded() && maps.loading.is_empty() {
            next_state.set(GameplayScreenState::Base);
            eprintln!("despawn_loading 1");
        }
    } else {
        *counter += 1;
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn despawn_loading(
    mut commands: Commands,
    root_entities: Query<Entity, With<LoadingRoot>>,
) {
    if let Ok(root_entity) = root_entities.get_single() {
        commands.entity(root_entity).despawn_recursive();
    }
    eprintln!("despawn_loading 2");
}
