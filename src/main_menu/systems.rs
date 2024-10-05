use crate::common::{log_if_slow, AssetPaths};
use crate::gameplay::{ActiveSav, GameplaySession};
use crate::hud::{
    trigger_button_action, ButtonBuilder, Fonts, RunButtonContext, BAD_TEXT_COLOR, GOOD_TEXT_COLOR,
    HARD_TEXT_COLOR, LARGE_SPACING, MEDIUM_SPACING, PANEL_COLOR,
};
use crate::keyboard::KeyBindings;
use crate::main_menu::components::{LoadButtonArea, MessageField, MessageWrapper};
use crate::main_menu::load_error::LoadError;
use crate::{application::ApplicationState, loading::ProgressScreenState, manual::ManualSection};
use base64::{engine::general_purpose::STANDARD as base64, Engine};
use bevy::prelude::{
    AlignContent, AlignItems, BuildChildren, Camera2dBundle, ChildBuilder, Commands,
    DespawnRecursiveExt, Display, Entity, Events, FlexDirection, FlexWrap, In, JustifyContent,
    Local, NextState, NodeBundle, Query, Res, ResMut, StateScoped, Style, Text, TextBundle, UiRect,
    Val, With, Without, World, ZIndex,
};
use bevy::{app::AppExit, ecs::system::SystemId};
use glob::glob;
use std::path::{Path, PathBuf};
use std::{cell::OnceCell, str::from_utf8, time::Instant};

const FULL_WIDTH: f32 = 720.0;

#[derive(Clone)]
pub(super) struct FoundSav(PathBuf);

impl RunButtonContext for FoundSav {}

#[derive(Clone, Debug)]
pub(super) struct LoadSystem(SystemId<FoundSav, ()>);

#[derive(Clone, Debug)]
pub(super) struct TriggerLoadSystem(SystemId<Entity, ()>);

#[allow(clippy::needless_pass_by_value)]
pub(super) fn create_button_systems(
    world: &mut World,
    load_system: Local<OnceCell<LoadSystem>>,
    trigger_load_system: Local<OnceCell<TriggerLoadSystem>>,
) -> (LoadSystem, TriggerLoadSystem) {
    (
        load_system
            .get_or_init(|| LoadSystem(world.register_system(load)))
            .clone(),
        trigger_load_system
            .get_or_init(|| {
                TriggerLoadSystem(world.register_system(trigger_button_action::<FoundSav>))
            })
            .clone(),
    )
}

#[derive(Clone, Debug)]
pub(super) struct QuitSystem(SystemId<(), ()>);

#[allow(clippy::needless_pass_by_value)]
pub(super) fn create_quit_system(
    world: &mut World,
    quit_system: Local<OnceCell<QuitSystem>>,
) -> QuitSystem {
    quit_system
        .get_or_init(|| QuitSystem(world.register_system(quit)))
        .clone()
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn spawn_main_menu(
    In(quit_system): In<QuitSystem>,
    mut commands: Commands,
    fonts: Res<Fonts>,
) {
    commands.spawn((
        Camera2dBundle::default(),
        StateScoped(ApplicationState::MainMenu),
    ));

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Style::default()
                },
                z_index: ZIndex::Global(3),
                ..NodeBundle::default()
            },
            StateScoped(ApplicationState::MainMenu),
        ))
        .with_children(|parent| {
            add_title(parent, &fonts);
            add_tagline(parent, &fonts);
            add_load_button_area(parent);
            add_notification_area(parent, &fonts);
            add_quit_button(parent, &quit_system, &fonts);
        });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn create_main_menu_key_bindings(
    world: &mut World,
    bindings: Local<KeyBindings<ApplicationState, (), ()>>,
) {
    let start = Instant::now();

    bindings.spawn(
        world,
        ApplicationState::MainMenu,
        |_| {},
        ManualSection::new(&[("load save", "a-z")], 100),
    );

    log_if_slow("create_main_menu_key_bindings", start);
}

fn add_title(parent: &mut ChildBuilder, fonts: &Fonts) {
    parent.spawn(TextBundle::from_section(
        "Rustaclysm",
        fonts.huge(HARD_TEXT_COLOR),
    ));
}

fn add_tagline(parent: &mut ChildBuilder, fonts: &Fonts) {
    parent.spawn(
        TextBundle::from_section(
            "A 3D reimplementation of Cataclysm: Dark Days Ahead",
            fonts.largish(HARD_TEXT_COLOR),
        )
        .with_style(Style {
            margin: UiRect {
                bottom: LARGE_SPACING,
                ..UiRect::default()
            },
            ..Style::default()
        }),
    );
}

fn add_load_button_area(parent: &mut ChildBuilder) {
    parent.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                height: Val::Px(440.0),
                align_items: AlignItems::Center,
                flex_wrap: FlexWrap::Wrap,
                align_content: AlignContent::Center,
                column_gap: MEDIUM_SPACING,
                ..Style::default()
            },
            ..NodeBundle::default()
        },
        LoadButtonArea,
    ));
}

fn add_notification_area(parent: &mut ChildBuilder, fonts: &Fonts) {
    parent
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(FULL_WIDTH),
                    height: MEDIUM_SPACING * 10.0,
                    align_items: AlignItems::FlexStart,
                    justify_content: JustifyContent::FlexStart,
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(MEDIUM_SPACING),
                    margin: UiRect {
                        bottom: MEDIUM_SPACING,
                        ..UiRect::default()
                    },
                    display: Display::None,
                    ..Style::default()
                },
                background_color: PANEL_COLOR.into(),
                ..NodeBundle::default()
            },
            MessageWrapper,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text::from_section("", fonts.largish(HARD_TEXT_COLOR)),
                    style: Style {
                        width: Val::Px(FULL_WIDTH),
                        padding: UiRect::horizontal(MEDIUM_SPACING),
                        flex_wrap: FlexWrap::Wrap,
                        ..Style::default()
                    },
                    ..TextBundle::default()
                },
                MessageField,
            ));
        });
}

fn add_quit_button(parent: &mut ChildBuilder, quit_system: &QuitSystem, fonts: &Fonts) {
    ButtonBuilder::new("Quit", fonts.large(BAD_TEXT_COLOR), quit_system.0)
        .large()
        .spawn(parent, ());
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn update_sav_files(
    In((load_system, trigger_load_system)): In<(LoadSystem, TriggerLoadSystem)>,
    mut commands: Commands,
    fonts: Res<Fonts>,
    mut session: GameplaySession,
    mut last_list_saves_result: Local<Option<Result<Vec<PathBuf>, LoadError>>>,
    mut load_button_areas: Query<
        (Entity, &mut Style),
        (With<LoadButtonArea>, Without<MessageWrapper>),
    >,
    mut message_wrappers: Query<&mut Style, (With<MessageWrapper>, Without<LoadButtonArea>)>,
    mut message_fields: Query<&mut Text, With<MessageField>>,
) {
    let mut message_wrapper_style = message_wrappers.single_mut();
    let (load_button_area, mut load_button_area_style) = load_button_areas.single_mut();

    let list_saves_result = list_saves();

    if !session.is_changed() && Some(&list_saves_result) == last_list_saves_result.as_ref() {
        return;
    }
    *last_list_saves_result = Some(list_saves_result.clone());

    commands.entity(load_button_area).despawn_descendants();

    match list_saves_result {
        Ok(list) => {
            load_button_area_style.display = Display::Flex;
            message_wrapper_style.display = Display::None;

            for (index, path) in list.iter().enumerate() {
                commands.entity(load_button_area).with_children(|parent| {
                    add_load_button(
                        &fonts,
                        &load_system,
                        &trigger_load_system,
                        parent,
                        path,
                        u32::try_from(index).ok(),
                    );
                });
            }
        }
        Err(err) => {
            eprintln!("{}", &err);

            load_button_area_style.display = Display::None;
            message_wrapper_style.display = Display::Flex;

            let mut message_text = message_fields.single_mut();
            message_text.sections[0].value = err.to_string();
        }
    }
}

fn list_saves() -> Result<Vec<PathBuf>, LoadError> {
    check_directory_structure()?;

    let worlds_pattern = AssetPaths::save().join("*");
    let pattern = worlds_pattern
        .to_str()
        .expect("Path pattern should be valid UTF-8");
    let worlds = glob(pattern)
        .expect("Paths shuld be readable")
        .map(|world| {
            world
                .expect("Path should be valid")
                .components()
                .skip(2)
                .collect::<PathBuf>()
        })
        .next();

    if worlds.is_none() {
        Err(LoadError::new(
            format!(
                "No Cataclysm: DDA worlds found to load under {}\nCreate a new world using Cataclysm: DDA to continue.",
                AssetPaths::save().display()
            )
        ))
    } else {
        let savs_pattern = worlds_pattern.join("#*.sav");
        let pattern = savs_pattern
            .to_str()
            .expect("Path pattern should be valid UTF-8");
        let savs = glob(pattern)
            .expect("Paths shuld be readable")
            .map(|sav| {
                sav.expect("Path should be valid")
                    .components()
                    .skip(2)
                    .collect::<PathBuf>()
            })
            .collect::<Vec<_>>();

        if savs.is_empty() {
            Err(LoadError::new(
                format!(
                    "No Cataclysm: DDA saves found to load in any world directory under {}\nCreate a new save file using Cataclysm: DDA to continue.",
                    AssetPaths::save().display()
                )
            ))
        } else {
            Ok(savs)
        }
    }
}

fn check_directory_structure() -> Result<(), LoadError> {
    if !AssetPaths::assets().is_dir() {
        return Err(LoadError::new(
            format!("Directory '{}' not found.\nPlease run this application in the directory containing the 'assets' directory.", AssetPaths::assets().display())
        ));
    }

    for asset_subdir in [AssetPaths::data(), AssetPaths::gfx(), AssetPaths::save()] {
        if !asset_subdir.is_dir() {
            return Err(LoadError::new(
                format!("Directory '{}/' not found.\nPlease make sure the '{}/' directory contains a copy of (or a symlink to) Cataclysm-DDA's '{}/' directory.", asset_subdir.display(), AssetPaths::assets().display(), asset_subdir.file_name().expect("Named directory").to_str().expect("Valid path"))
            ));
        }
    }

    Ok(())
}

fn add_load_button(
    fonts: &Fonts,
    load_system: &LoadSystem,
    trigger_load_system: &TriggerLoadSystem,
    parent: &mut ChildBuilder,
    path: &Path,
    index: Option<u32>,
) {
    let world_path = path.parent().expect("World required");
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
    let character = from_utf8(&decoded_character).expect("Valid utf8 required");

    let mut key_binding = None;
    if let Some(index) = index {
        if index <= 26 {
            key_binding =
                Some(char::from_u32('a' as u32 + index).expect("Valid unicode character (a-z)"));
        }
    }

    ButtonBuilder::new(
        format!("Load {character} in {}", world_path.display()),
        fonts.largish(GOOD_TEXT_COLOR),
        load_system.0,
    )
    .with_style(Style {
        width: Val::Px(400.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        margin: UiRect {
            bottom: MEDIUM_SPACING,
            ..UiRect::default()
        },
        padding: UiRect {
            left: LARGE_SPACING,
            right: LARGE_SPACING,
            top: MEDIUM_SPACING,
            bottom: MEDIUM_SPACING,
        },
        ..Style::default()
    })
    .key_binding(key_binding, trigger_load_system.0)
    .spawn(parent, FoundSav(path.to_path_buf()));
}

pub(super) fn load(
    In(found_sav): In<FoundSav>,
    mut commands: Commands,
    mut next_progress_state: ResMut<NextState<ProgressScreenState>>,
) {
    commands.insert_resource(ActiveSav::new(&found_sav.0));
    next_progress_state.set(ProgressScreenState::Loading);
}

fn quit(mut app_exit_events: ResMut<Events<AppExit>>) {
    app_exit_events.send(AppExit::Success);
}
