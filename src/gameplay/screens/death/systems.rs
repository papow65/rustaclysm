use crate::common::log_if_slow;
use crate::hud::{
    ButtonBuilder, Fonts, BAD_TEXT_COLOR, PANEL_COLOR, SMALL_SPACING, WARN_TEXT_COLOR,
};
use crate::keyboard::KeyBindings;
use crate::manual::ManualSection;
use crate::{application::ApplicationState, gameplay::GameplayScreenState};
use bevy::ecs::system::SystemId;
use bevy::prelude::{
    AlignItems, BuildChildren, Commands, FlexDirection, In, JustifyContent, KeyCode, Local,
    NextState, NodeBundle, Res, ResMut, StateScoped, Style, TextBundle, UiRect, Val, World,
};
use std::{cell::OnceCell, time::Instant};

#[derive(Clone, Debug)]
pub(super) struct MainMenuSystem(SystemId<(), ()>);

#[allow(clippy::needless_pass_by_value)]
pub(super) fn create_main_menu_system(
    world: &mut World,
    main_menu_system: Local<OnceCell<MainMenuSystem>>,
) -> MainMenuSystem {
    main_menu_system
        .get_or_init(|| MainMenuSystem(world.register_system(to_main_menu)))
        .clone()
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn spawn_death_screen(
    In(main_menu_system): In<MainMenuSystem>,
    mut commands: Commands,
    fonts: Res<Fonts>,
) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Style::default()
                },
                ..NodeBundle::default()
            },
            StateScoped(GameplayScreenState::Death),
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(300.0),
                        height: Val::Px(200.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(SMALL_SPACING),
                        ..Style::default()
                    },
                    background_color: PANEL_COLOR.into(),
                    ..NodeBundle::default()
                })
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
                            ..NodeBundle::default()
                        })
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "You died",
                                fonts.largish(BAD_TEXT_COLOR),
                            ));
                        });

                    ButtonBuilder::new(
                        "Main menu",
                        fonts.regular(WARN_TEXT_COLOR),
                        main_menu_system.0,
                    )
                    .large()
                    .spawn(parent, ());
                });
        });
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn create_death_screen_key_bindings(
    world: &mut World,
    bindings: Local<KeyBindings<GameplayScreenState, (), ()>>,
) {
    let start = Instant::now();

    bindings.spawn(
        world,
        GameplayScreenState::Death,
        |bindings| {
            bindings.add_multi(
                [KeyCode::Escape, KeyCode::Enter, KeyCode::Space],
                to_main_menu_wrapper,
            );
        },
        ManualSection::new(&[("to main menu", "esc/enter/space")], 100),
    );

    log_if_slow("create_death_screen_key_bindings", start);
}

fn to_main_menu_wrapper(next_application_state: ResMut<NextState<ApplicationState>>) {
    to_main_menu(next_application_state);
}

fn to_main_menu(mut next_application_state: ResMut<NextState<ApplicationState>>) {
    next_application_state.set(ApplicationState::MainMenu);
}
