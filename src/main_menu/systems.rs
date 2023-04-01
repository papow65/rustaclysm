use crate::prelude::*;
use bevy::{
    app::AppExit,
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};

const FULL_WIDTH: f32 = 720.0;
const SPACING: f32 = 20.0;

const LOAD_FOREGROUND: Color = Color::rgb(0.35, 0.75, 0.35);
const QUIT_FOREGROUND: Color = Color::rgb(0.75, 0.35, 0.35);
const TEXT_FOREGROUND: Color = Color::rgb(1.0, 1.0, 1.0);

const NORMAL_BACKGROUND: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BACKGROUND: Color = Color::rgb(0.25, 0.25, 0.25);

fn font(asset_server: &AssetServer) -> Handle<Font> {
    asset_server.load("assets/fonts/FiraMono-Medium.otf")
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                size: Size::width(Val::Percent(100.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Title
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::width(Val::Px(FULL_WIDTH)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect {
                            bottom: Val::Px(SPACING),
                            ..UiRect::default()
                        },
                        ..default()
                    },
                    //background_color: NORMAL_BACKGROUND.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Rustaclysm",
                        TextStyle {
                            font: font(&asset_server),
                            font_size: 120.0,
                            color: TEXT_FOREGROUND,
                        },
                    ));
                });

            // Load buttons
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        size: Size::new(Val::Percent(100.0), Val::Px(400.0)),
                        align_items: AlignItems::Center,
                        flex_wrap: FlexWrap::Wrap,
                        align_content: AlignContent::Center,
                        gap: Size::width(Val::Px(SPACING)),
                        ..default()
                    },
                    ..default()
                })
                .insert(LoadButtonArea);

            // Spacing here is handled by the children of LoadButtonArea

            // Notification area
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(FULL_WIDTH), Val::Px(10.0 * SPACING)),
                        align_items: AlignItems::FlexStart,
                        justify_content: JustifyContent::FlexStart,
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(SPACING)),
                        margin: UiRect {
                            bottom: Val::Px(SPACING),
                            ..UiRect::default()
                        },
                        ..default()
                    },
                    background_color: NORMAL_BACKGROUND.into(),
                    ..default()
                })
                .insert(MessageWrapper)
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle {
                            text: Text::from_section(
                                "",
                                TextStyle {
                                    font: font(&asset_server),
                                    font_size: 20.0,
                                    color: TEXT_FOREGROUND,
                                },
                            ),
                            style: Style {
                                size: Size::width(Val::Px(FULL_WIDTH - 2.0 * SPACING)),
                                flex_wrap: FlexWrap::Wrap,
                                ..Style::default()
                            },
                            ..TextBundle::default()
                        })
                        .insert(MessageField);
                });

            // Quit button
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(200.0), Val::Px(70.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: NORMAL_BACKGROUND.into(),
                    ..default()
                })
                .insert(QuitButton)
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Quit",
                        TextStyle {
                            font: font(&asset_server),
                            font_size: 40.0,
                            color: QUIT_FOREGROUND,
                        },
                    ));
                });
        });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_main_menu_button_input(
    mut commands: Commands,
    mut next_state: ResMut<NextState<ApplicationState>>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&LoadButton>,
            Option<&QuitButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, load_button, quit_button) in &mut interaction_query {
        match (*interaction, load_button, quit_button.is_some()) {
            (Interaction::Clicked, Some(load_button), false) => {
                commands.insert_resource(Paths::new(&load_button.path));
                next_state.set(ApplicationState::Gameplay);
            }
            (Interaction::Clicked, None, true) => app_exit_events.send(AppExit),
            (Interaction::Clicked, play, quit) => {
                panic!("{play:?} {quit:?}");
            }
            (Interaction::Hovered, ..) => {
                *color = HOVERED_BACKGROUND.into();
            }
            (Interaction::None, ..) => {
                *color = NORMAL_BACKGROUND.into();
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_sav_files(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut last_error: Local<Option<LoadError>>,
    load_button_areas: Query<Entity, With<LoadButtonArea>>,
    mut message_wrappers: Query<&mut Style, With<MessageWrapper>>,
    mut message_fields: Query<&mut Text, With<MessageField>>,
) {
    if let Ok(mut message_wrapper_style) = message_wrappers.get_single_mut() {
        if let Ok(mut message_text) = message_fields.get_single_mut() {
            if let Ok(load_button_area) = load_button_areas.get_single() {
                match Paths::list() {
                    Ok(list) => {
                        *last_error = None;
                        message_wrapper_style.display = Display::None;

                        commands.entity(load_button_area).despawn_descendants();

                        for path in list {
                            commands.entity(load_button_area).with_children(|parent| {
                                // Load button
                                parent
                                    .spawn(ButtonBundle {
                                        style: Style {
                                            size: Size::width(Val::Px(400.0)),
                                            align_items: AlignItems::Center,
                                            margin: UiRect {
                                                bottom: Val::Px(SPACING),
                                                ..UiRect::default()
                                            },
                                            padding: UiRect {
                                                left: Val::Px(2.0 * SPACING),
                                                right: Val::Px(2.0 * SPACING),
                                                top: Val::Px(SPACING),
                                                bottom: Val::Px(SPACING),
                                            },
                                            ..default()
                                        },
                                        background_color: NORMAL_BACKGROUND.into(),
                                        ..default()
                                    })
                                    .insert(LoadButton { path: path.clone() })
                                    .with_children(|parent| {
                                        parent.spawn(TextBundle::from_section(
                                            format!("Load {}", path.display()),
                                            TextStyle {
                                                font: font(&asset_server),
                                                font_size: 20.0,
                                                color: LOAD_FOREGROUND,
                                            },
                                        ));
                                    });
                            });
                        }
                    }
                    Err(err) => {
                        if Some(&err) != last_error.as_ref() {
                            eprintln!("{}", &err);
                            *last_error = Some(err.clone());
                        }

                        if let Ok(load_button_area) = load_button_areas.get_single() {
                            commands.entity(load_button_area).despawn_descendants();
                        }

                        message_wrapper_style.display = Display::Flex;
                        message_text.sections[0].value = err.to_string();
                    }
                }
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_main_menu_keyboard_input(
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
    mut key_events: EventReader<KeyboardInput>,
    mut next_state: ResMut<NextState<ApplicationState>>,
) {
    for key_event in key_events.iter() {
        if key_event.state != ButtonState::Pressed {
            continue;
        }

        match key_event.key_code {
            Some(KeyCode::C | KeyCode::D | KeyCode::Q) => app_exit_events.send(AppExit),
            Some(KeyCode::P | KeyCode::Space | KeyCode::Return) => {
                next_state.set(ApplicationState::Gameplay);
            }
            _ => {}
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn despawn_main_menu(
    mut commands: Commands,
    root_entities: Query<Entity, Or<(With<Camera>, With<Node>)>>,
) {
    for root_entity in root_entities.iter() {
        commands.entity(root_entity).despawn_recursive();
    }
}
