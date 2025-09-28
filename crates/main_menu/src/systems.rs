use crate::{LoadButtonArea, LoadError, LogMessageField, LogMessageWrapper};
use application_state::ApplicationState;
use base64::{Engine as _, engine::general_purpose::STANDARD as base64};
use bevy::ecs::{spawn::SpawnIter, system::SystemId};
use bevy::prelude::{
    AlignContent, AlignItems, AppExit, Bundle, Children, Commands, DespawnOnExit, Display, Entity,
    FlexDirection, FlexWrap, GlobalZIndex, In, JustifyContent, Messages, NextState, Node, Res,
    ResMut, Single, SpawnRelated as _, Text, TextFont, UiRect, Val, With, Without, World, children,
    debug, error,
};
use gameplay_cdda_active_sav::ActiveSav;
use gameplay_local::GameplayLocal;
use glob::glob;
use hud::{
    BAD_TEXT_COLOR, ButtonBuilder, Fonts, GOOD_TEXT_COLOR, HARD_TEXT_COLOR, LARGE_SPACING,
    MEDIUM_SPACING, PANEL_COLOR, trigger_button_action,
};
use manual::ManualSection;
use std::path::{Path, PathBuf};
use std::{str::from_utf8, time::Instant};
use util::{AssetPaths, log_if_slow};

const FULL_WIDTH: f32 = 720.0;

pub(super) fn enter_main_menu(mut next_application_state: ResMut<NextState<ApplicationState>>) {
    next_application_state.set(ApplicationState::MainMenu);
}

#[derive(Clone, Debug)]
pub(super) struct FoundSav(PathBuf);

#[derive(Debug)]
pub(super) struct LoadSystems {
    button: SystemId<In<FoundSav>, ()>,
    key: SystemId<In<Entity>, ()>,
}

pub(super) fn create_load_systems(world: &mut World) -> LoadSystems {
    LoadSystems {
        button: world.register_system_cached(load),
        key: world.register_system_cached(trigger_button_action::<In<FoundSav>>),
    }
}

#[derive(Debug)]
pub(super) struct QuitSystem(SystemId<(), ()>);

pub(super) fn create_quit_system(world: &mut World) -> QuitSystem {
    QuitSystem(world.register_system_cached(quit))
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn spawn_main_menu(
    In(quit_system): In<QuitSystem>,
    mut commands: Commands,
    fonts: Res<Fonts>,
) {
    commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..Node::default()
        },
        GlobalZIndex(3),
        DespawnOnExit(ApplicationState::MainMenu),
        children![
            title(&fonts),
            tagline(&fonts),
            load_button_area(),
            notification_area(&fonts),
            quit_button(&quit_system, &fonts),
        ],
    ));
}

pub(crate) fn create_main_menu_key_bindings(world: &mut World) {
    let start = Instant::now();

    world.spawn((
        ManualSection::new(&[("load save", "a-z")], 100),
        DespawnOnExit(ApplicationState::MainMenu),
    ));

    log_if_slow("create_main_menu_key_bindings", start);
}

fn title(fonts: &Fonts) -> impl Bundle {
    (Text::from("Rustaclysm"), HARD_TEXT_COLOR, fonts.huge())
}

fn tagline(fonts: &Fonts) -> impl Bundle {
    (
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
    )
}

fn load_button_area() -> impl Bundle {
    (
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
    )
}

fn notification_area(fonts: &Fonts) -> impl Bundle {
    (
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
        LogMessageWrapper,
        children![(
            Text::default(),
            HARD_TEXT_COLOR,
            fonts.largish(),
            Node {
                width: Val::Px(FULL_WIDTH),
                padding: UiRect::horizontal(MEDIUM_SPACING),
                flex_wrap: FlexWrap::Wrap,
                ..Node::default()
            },
            LogMessageField,
        )],
    )
}

fn quit_button(quit_system: &QuitSystem, fonts: &Fonts) -> impl Bundle {
    ButtonBuilder::new("Quit", BAD_TEXT_COLOR, fonts.large(), quit_system.0, ())
        .large()
        .bundle()
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn update_sav_files(
    In(load_systems): In<LoadSystems>,
    mut commands: Commands,
    fonts: Res<Fonts>,
    mut last_list_saves_result: GameplayLocal<Option<Result<Vec<PathBuf>, LoadError>>>,
    mut load_button_areas: Single<
        (Entity, &mut Node),
        (With<LoadButtonArea>, Without<LogMessageWrapper>),
    >,
    mut message_wrapper: Single<&mut Node, (With<LogMessageWrapper>, Without<LoadButtonArea>)>,
    mut message_field: Single<&mut Text, With<LogMessageField>>,
) {
    let start = Instant::now();

    let &mut (load_button_area, ref mut load_button_area_style) = &mut *load_button_areas;

    let list_saves_result = list_saves();

    if Some(&list_saves_result) == last_list_saves_result.get().as_ref() {
        return;
    }
    *last_list_saves_result.get() = Some(list_saves_result.clone());

    commands
        .entity(load_button_area)
        .despawn_related::<Children>();

    match list_saves_result {
        Ok(list) => {
            load_button_area_style.display = Display::Flex;
            message_wrapper.display = Display::None;

            let largish = fonts.largish();

            commands
                .entity(load_button_area)
                .insert(Children::spawn((SpawnIter(
                    list.into_iter().enumerate().map(move |(index, path)| {
                        load_button(
                            largish.clone(),
                            &load_systems,
                            &path,
                            u32::try_from(index).ok(),
                        )
                    }),
                ),)));
        }
        Err(err) => {
            error!("{}", &err);

            load_button_area_style.display = Display::None;
            message_wrapper.display = Display::Flex;

            message_field.0 = err.to_string();
        }
    }

    debug!("Updated main menu sav files in {:?}", start.elapsed());
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

fn load_button(
    largish: TextFont,
    load_systems: &LoadSystems,
    path: &Path,
    index: Option<u32>,
) -> impl Bundle + use<> {
    const ALPHABET: [char; 26] = {
        let mut alphabet = ['\0'; 26];
        let mut i = 0;
        while i < alphabet.len() {
            alphabet[i] = (b'a' + i as u8) as char;
            i += 1;
        }

        alphabet
    };
    let key_binding = index.and_then(|index| ALPHABET.get(index as usize).copied());

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

    ButtonBuilder::new(
        format!("Load {character} in {}", world_path.display()),
        GOOD_TEXT_COLOR,
        largish,
        load_systems.button,
        FoundSav(path.to_path_buf()),
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
    .bundle()
}

pub(super) fn load(
    In(found_sav): In<FoundSav>,
    mut commands: Commands,
    mut next_application_state: ResMut<NextState<ApplicationState>>,
) {
    commands.insert_resource(
        ActiveSav::new(&found_sav.0).expect("Loading sav file should not have failed"),
    );
    next_application_state.set(ApplicationState::PreGameplay);
}

fn quit(mut app_exit_events: ResMut<Messages<AppExit>>) {
    app_exit_events.write(AppExit::Success);
}
