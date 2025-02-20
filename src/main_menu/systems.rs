use crate::gameplay::{ActiveSav, GameplayLocal};
use crate::hud::{
    BAD_TEXT_COLOR, ButtonBuilder, Fonts, GOOD_TEXT_COLOR, HARD_TEXT_COLOR, LARGE_SPACING,
    MEDIUM_SPACING, PANEL_COLOR, trigger_button_action,
};
use crate::keyboard::KeyBindings;
use crate::main_menu::components::{LoadButtonArea, MessageField, MessageWrapper};
use crate::main_menu::load_error::LoadError;
use crate::util::{AssetPaths, log_if_slow};
use crate::{application::ApplicationState, manual::ManualSection};
use base64::{Engine as _, engine::general_purpose::STANDARD as base64};
use bevy::prelude::{
    AlignContent, AlignItems, BuildChildren as _, Camera2d, ChildBuild as _, ChildBuilder,
    Commands, DespawnRecursiveExt as _, Display, Entity, Events, FlexDirection, FlexWrap,
    GlobalZIndex, In, JustifyContent, Local, NextState, Node, Query, Res, ResMut, StateScoped,
    Text, UiRect, Val, With, Without, World,
};
use bevy::{app::AppExit, ecs::system::SystemId};
use glob::glob;
use std::path::{Path, PathBuf};
use std::{str::from_utf8, time::Instant};

pub(super) fn enter_main_menu(mut next_application_state: ResMut<NextState<ApplicationState>>) {
    next_application_state.set(ApplicationState::MainMenu);
}

const FULL_WIDTH: f32 = 720.0;

#[derive(Clone, Debug)]
pub(super) struct FoundSav(PathBuf);

#[derive(Debug)]
pub(super) struct LoadSystems {
    button: SystemId<In<FoundSav>, ()>,
    key: SystemId<In<Entity>, ()>,
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn create_load_systems(world: &mut World) -> LoadSystems {
    LoadSystems {
        button: world.register_system_cached(load),
        key: world.register_system_cached(trigger_button_action::<In<FoundSav>>),
    }
}

#[derive(Debug)]
pub(super) struct QuitSystem(SystemId<(), ()>);

#[allow(clippy::needless_pass_by_value)]
pub(super) fn create_quit_system(world: &mut World) -> QuitSystem {
    QuitSystem(world.register_system_cached(quit))
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn spawn_main_menu(
    In(quit_system): In<QuitSystem>,
    mut commands: Commands,
    fonts: Res<Fonts>,
) {
    commands.spawn((Camera2d, StateScoped(ApplicationState::MainMenu)));

    commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Node::default()
            },
            GlobalZIndex(3),
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
    parent.spawn((Text::from("Rustaclysm"), HARD_TEXT_COLOR, fonts.huge()));
}

fn add_tagline(parent: &mut ChildBuilder, fonts: &Fonts) {
    parent.spawn((
        Text::from("A 3D reimplementation of Cataclysm: Dark Days Ahead"),
        HARD_TEXT_COLOR,
        fonts.largish(),
        Node {
            margin: UiRect {
                bottom: LARGE_SPACING,
                ..UiRect::default()
            },
            ..Node::default()
        },
    ));
}

fn add_load_button_area(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            height: Val::Px(440.0),
            align_items: AlignItems::Center,
            flex_wrap: FlexWrap::Wrap,
            align_content: AlignContent::Center,
            column_gap: MEDIUM_SPACING,
            ..Node::default()
        },
        LoadButtonArea,
    ));
}

fn add_notification_area(parent: &mut ChildBuilder, fonts: &Fonts) {
    parent
        .spawn((
            Node {
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
                ..Node::default()
            },
            PANEL_COLOR,
            MessageWrapper,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::default(),
                HARD_TEXT_COLOR,
                fonts.largish(),
                Node {
                    width: Val::Px(FULL_WIDTH),
                    padding: UiRect::horizontal(MEDIUM_SPACING),
                    flex_wrap: FlexWrap::Wrap,
                    ..Node::default()
                },
                MessageField,
            ));
        });
}

fn add_quit_button(parent: &mut ChildBuilder, quit_system: &QuitSystem, fonts: &Fonts) {
    ButtonBuilder::new("Quit", BAD_TEXT_COLOR, fonts.large(), quit_system.0)
        .large()
        .spawn(parent, ());
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn update_sav_files(
    In(load_systems): In<LoadSystems>,
    mut commands: Commands,
    fonts: Res<Fonts>,
    mut last_list_saves_result: GameplayLocal<Option<Result<Vec<PathBuf>, LoadError>>>,
    mut load_button_areas: Query<
        (Entity, &mut Node),
        (With<LoadButtonArea>, Without<MessageWrapper>),
    >,
    mut message_wrappers: Query<&mut Node, (With<MessageWrapper>, Without<LoadButtonArea>)>,
    mut message_fields: Query<&mut Text, With<MessageField>>,
) {
    let mut message_wrapper_style = message_wrappers.single_mut();
    let (load_button_area, mut load_button_area_style) = load_button_areas.single_mut();

    let list_saves_result = list_saves();

    if Some(&list_saves_result) == last_list_saves_result.get().as_ref() {
        return;
    }
    *last_list_saves_result.get() = Some(list_saves_result.clone());

    commands.entity(load_button_area).despawn_descendants();

    match list_saves_result {
        Ok(list) => {
            load_button_area_style.display = Display::Flex;
            message_wrapper_style.display = Display::None;

            for (index, path) in list.iter().enumerate() {
                commands.entity(load_button_area).with_children(|parent| {
                    add_load_button(
                        &fonts,
                        &load_systems,
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
            message_text.0 = err.to_string();
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
        Err(LoadError::new(format!(
            "No Cataclysm: DDA worlds found to load under {}\nCreate a new world using Cataclysm: DDA to continue.",
            AssetPaths::save().display()
        )))
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
            Err(LoadError::new(format!(
                "No Cataclysm: DDA saves found to load in any world directory under {}\nCreate a new save file using Cataclysm: DDA to continue.",
                AssetPaths::save().display()
            )))
        } else {
            Ok(savs)
        }
    }
}

fn check_directory_structure() -> Result<(), LoadError> {
    if !AssetPaths::assets().is_dir() {
        return Err(LoadError::new(format!(
            "Directory '{}' not found.\nPlease run this application in the directory containing the 'assets' directory.",
            AssetPaths::assets().display()
        )));
    }

    for asset_subdir in [AssetPaths::data(), AssetPaths::gfx(), AssetPaths::save()] {
        if !asset_subdir.is_dir() {
            return Err(LoadError::new(format!(
                "Directory '{}/' not found.\nPlease make sure the '{}/' directory contains a copy of (or a symlink to) Cataclysm-DDA's '{}/' directory.",
                asset_subdir.display(),
                AssetPaths::assets().display(),
                asset_subdir
                    .file_name()
                    .expect("Named directory")
                    .to_str()
                    .expect("Valid path")
            )));
        }
    }

    Ok(())
}

fn add_load_button(
    fonts: &Fonts,
    load_systems: &LoadSystems,
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
        GOOD_TEXT_COLOR,
        fonts.largish(),
        load_systems.button,
    )
    .with_node(Node {
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
        ..Node::default()
    })
    .key_binding(key_binding, load_systems.key)
    .spawn(parent, FoundSav(path.to_path_buf()));
}

pub(super) fn load(
    In(found_sav): In<FoundSav>,
    mut commands: Commands,
    mut next_application_state: ResMut<NextState<ApplicationState>>,
) {
    commands.insert_resource(ActiveSav::new(&found_sav.0));
    next_application_state.set(ApplicationState::PreGameplay);
}

fn quit(mut app_exit_events: ResMut<Events<AppExit>>) {
    app_exit_events.send(AppExit::Success);
}
