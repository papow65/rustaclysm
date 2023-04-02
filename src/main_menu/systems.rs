use crate::prelude::*;
use base64::{engine::general_purpose::STANDARD as base64, Engine};
use bevy::{
    app::AppExit,
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use std::str::from_utf8;

const FULL_WIDTH: f32 = 720.0;
const SPACING: f32 = 20.0;

const BACKGROUND_WIDTH: f32 = 1552.0;
const BACKGROUND_HEIGHT: f32 = 1009.0;
const BACKGROUND_NAME: &str = "on_the_run.png";

const LOAD_FOREGROUND: Color = Color::rgb(0.35, 0.75, 0.35);
const QUIT_FOREGROUND: Color = Color::rgb(0.75, 0.35, 0.35);
const TEXT_FOREGROUND: Color = Color::rgb(1.0, 1.0, 1.0);

const NORMAL_BACKGROUND: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BACKGROUND: Color = Color::rgb(0.25, 0.25, 0.25);

fn font(asset_server: &AssetServer) -> Handle<Font> {
    asset_server.load(Paths::fonts_path().join("FiraMono-Medium.otf"))
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let background_image = asset_server.load(Paths::backgrounds_path().join(BACKGROUND_NAME));
    commands
        .spawn(SpriteBundle {
            texture: background_image.into(),
            ..Default::default()
        }).insert(Background);

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
            parent.spawn(TextBundle::from_section(
                "Rustaclysm",
                TextStyle {
                    font: font(&asset_server),
                    font_size: 120.0,
                    color: TEXT_FOREGROUND,
                },
            ));

            // Tagline
            parent.spawn(
                TextBundle::from_section(
                    "A 3D reimplementation of Cataclysm: Dark Days Ahead",
                    TextStyle {
                        font: font(&asset_server),
                        font_size: 22.0,
                        color: TEXT_FOREGROUND,
                    },
                )
                .with_style(Style {
                    margin: UiRect {
                        bottom: Val::Px(2.0 * SPACING),
                        ..UiRect::default()
                    },
                    ..default()
                }),
            );

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
pub(crate) fn update_sav_files(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut last_error: Local<Option<LoadError>>,
    mut load_button_areas: Query<(Entity, &mut Style), (With<LoadButtonArea>, Without<MessageWrapper>)>,
    mut message_wrappers: Query<&mut Style, (With<MessageWrapper>, Without<LoadButtonArea>)>,
    mut message_fields: Query<&mut Text, With<MessageField>>,
) {
    if let Ok(mut message_wrapper_style) = message_wrappers.get_single_mut() {
        if let Ok(mut message_text) = message_fields.get_single_mut() {
            if let Ok((load_button_area, mut load_button_area_style)) = load_button_areas.get_single_mut() {
                match Paths::list() {
                    Ok(list) => {
                        *last_error = None;
                        load_button_area_style.display = Display::Flex;
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
                                            justify_content: JustifyContent::Center,
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
                                        let world = path.parent().expect("World required");
                                        let encoded_character = path
                                            .file_name()
                                            .expect("Filename required")
                                            .to_str()
                                            .expect("Valid utf-8 filename required")
                                            .strip_prefix('#')
                                            .expect("Expected # prefix")
                                            .strip_suffix(".sav")
                                            .expect("Expected .sav suffix");
                                        let decoded_character = base64
                                            .decode(encoded_character)
                                            .expect("Valid base64 required");
                                        let character = from_utf8(&decoded_character)
                                            .expect("Valid utf8 required");

                                        parent.spawn(TextBundle::from_section(
                                            format!("Load {} in {}", character, world.display()),
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

                        commands.entity(load_button_area).despawn_descendants();

                        load_button_area_style.display = Display::None;
                        message_wrapper_style.display = Display::Flex;
                        message_text.sections[0].value = err.to_string();
                    }
                }
            }
        }
    }
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
pub(crate) fn resize_background(
    cameras: Query<&Camera>,
    mut backgrounds: Query<&mut Transform, With<Background>>
) {
    //for _ in resize_events.iter() {
        for camera in cameras.iter() {
            if let Some(camera_size) = &camera.physical_target_size() {
                let scale = (camera_size.x as f32 / BACKGROUND_WIDTH).max(camera_size.y as f32 / BACKGROUND_HEIGHT);

                for mut background in backgrounds.iter_mut() {
                    *background = Transform::from_scale(Vec3::new(scale, scale, 1.0));
                }
            }
        }
    //}
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