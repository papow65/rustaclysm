use crate::GameplayScreenState;
use application_state::ApplicationState;
use bevy::prelude::{
    AlignItems, Commands, Events, FlexDirection, In, JustifyContent, KeyCode, Local, NextState,
    Node, Res, ResMut, SpawnRelated as _, StateScoped, Val, World, children,
};
use bevy::{app::AppExit, ecs::system::SystemId};
use hud::{BAD_TEXT_COLOR, ButtonBuilder, Fonts, GOOD_TEXT_COLOR, HARD_TEXT_COLOR, MEDIUM_SPACING};
use keyboard::KeyBindings;
use manual::ManualSection;
use std::time::Instant;
use util::log_if_slow;

#[derive(Debug)]
pub(super) struct MenuButtonActions {
    return_: SystemId<(), ()>,
    main_menu: SystemId<(), ()>,
    quit: SystemId<(), ()>,
}

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
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: MEDIUM_SPACING,
            ..Node::default()
        },
        StateScoped(GameplayScreenState::Menu),
        children![
            ButtonBuilder::new(
                "Return",
                GOOD_TEXT_COLOR,
                fonts.large(),
                button_actions.return_,
                (),
            )
            .large()
            .bundle(),
            ButtonBuilder::new(
                "Main Menu",
                HARD_TEXT_COLOR,
                fonts.large(),
                button_actions.main_menu,
                (),
            )
            .large()
            .bundle(),
            ButtonBuilder::new(
                "Quit",
                BAD_TEXT_COLOR,
                fonts.large(),
                button_actions.quit,
                (),
            )
            .large()
            .bundle(),
        ],
    ));
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn create_menu_key_bindings(
    world: &mut World,
    bindings: Local<KeyBindings<GameplayScreenState, (), ()>>,
) {
    let start = Instant::now();

    bindings.spawn(world, GameplayScreenState::Menu, |bindings| {
        bindings.add(KeyCode::Escape, close_menu);
    });

    world.spawn((
        ManualSection::new(&[("close menu", "esc")], 100),
        StateScoped(GameplayScreenState::Menu),
    ));

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
