use crate::application::ApplicationState;
use crate::common::log_if_slow;
use crate::gameplay::GameplayScreenState;
use crate::hud::{
    ButtonBuilder, Fonts, BAD_TEXT_COLOR, GOOD_TEXT_COLOR, HARD_TEXT_COLOR, MEDIUM_SPACING,
};
use crate::keyboard::KeyBindings;
use crate::manual::ManualSection;
use bevy::prelude::{
    AlignItems, BuildChildren, Commands, Events, FlexDirection, In, JustifyContent, KeyCode, Local,
    NextState, NodeBundle, Res, ResMut, StateScoped, Style, Val, World,
};
use bevy::{app::AppExit, ecs::system::SystemId};
use std::time::Instant;

#[derive(Debug)]
pub(super) struct MenuButtonActions {
    return_: SystemId<(), ()>,
    main_menu: SystemId<(), ()>,
    quit: SystemId<(), ()>,
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn create_menu_button_actions(world: &mut World) -> MenuButtonActions {
    MenuButtonActions {
        return_: world.register_system_cached(return_),
        main_menu: world.register_system_cached(main_menu),
        quit: world.register_system_cached(quit),
    }
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn spawn_menu(
    In(button_actions): In<MenuButtonActions>,
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
                    row_gap: MEDIUM_SPACING,
                    ..Style::default()
                },
                ..NodeBundle::default()
            },
            StateScoped(GameplayScreenState::Menu),
        ))
        .with_children(|parent| {
            ButtonBuilder::new(
                "Return",
                GOOD_TEXT_COLOR,
                fonts.large(),
                button_actions.return_,
            )
            .large()
            .spawn(parent, ());
            ButtonBuilder::new(
                "Main Menu",
                HARD_TEXT_COLOR,
                fonts.large(),
                button_actions.main_menu,
            )
            .large()
            .spawn(parent, ());
            ButtonBuilder::new("Quit", BAD_TEXT_COLOR, fonts.large(), button_actions.quit)
                .large()
                .spawn(parent, ());
        });
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn create_menu_key_bindings(
    world: &mut World,
    bindings: Local<KeyBindings<GameplayScreenState, (), ()>>,
) {
    let start = Instant::now();

    bindings.spawn(
        world,
        GameplayScreenState::Menu,
        |bindings| {
            bindings.add(KeyCode::Escape, close_menu);
        },
        ManualSection::new(&[("close menu", "esc")], 100),
    );

    log_if_slow("create_menu_key_bindings", start);
}

fn close_menu(mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>) {
    next_gameplay_state.set(GameplayScreenState::Base);
}

fn return_(mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>) {
    next_gameplay_state.set(GameplayScreenState::Base);
}

pub(super) fn main_menu(mut next_application_state: ResMut<NextState<ApplicationState>>) {
    next_application_state.set(ApplicationState::MainMenu);
}

pub(super) fn quit(mut app_exit_events: ResMut<Events<AppExit>>) {
    app_exit_events.send(AppExit::Success);
}
