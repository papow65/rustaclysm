use crate::application::ApplicationState;
use crate::common::log_if_slow;
use crate::gameplay::screens::menu::components::{MainMenuButton, QuitButton, ReturnButton};
use crate::gameplay::GameplayScreenState;
use crate::hud::{
    ButtonBuilder, Fonts, BAD_TEXT_COLOR, GOOD_TEXT_COLOR, HARD_TEXT_COLOR, MEDIUM_SPACING,
};
use crate::keyboard::{Key, KeyBindings};
use crate::manual::ManualSection;
use bevy::app::AppExit;
use bevy::prelude::{
    AlignItems, BuildChildren, Button, Changed, Commands, Events, FlexDirection, In, Interaction,
    JustifyContent, KeyCode, Local, NextState, NodeBundle, Query, Res, ResMut, StateScoped, Style,
    Val, With, World,
};
use std::time::Instant;

#[expect(clippy::needless_pass_by_value)]
pub(super) fn spawn_menu(mut commands: Commands, fonts: Res<Fonts>) {
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
            ButtonBuilder::new("Return", fonts.large(GOOD_TEXT_COLOR), ReturnButton)
                .large()
                .spawn(parent);
            ButtonBuilder::new("Main Menu", fonts.large(HARD_TEXT_COLOR), MainMenuButton)
                .large()
                .spawn(parent);
            ButtonBuilder::new("Quit", fonts.large(BAD_TEXT_COLOR), QuitButton)
                .large()
                .spawn(parent);
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

fn close_menu(In(_): In<Key>, mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>) {
    next_gameplay_state.set(GameplayScreenState::Base);
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn manage_menu_button_input(
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    interaction_query: Query<
        (
            &Interaction,
            Option<&ReturnButton>,
            Option<&MainMenuButton>,
            Option<&QuitButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, return_button, main_menu_button, quit_button) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match (
                return_button.is_some(),
                main_menu_button.is_some(),
                quit_button.is_some(),
            ) {
                (true, false, false) => {
                    next_gameplay_state.set(GameplayScreenState::Base);
                }
                (false, true, false) => {
                    next_application_state.set(ApplicationState::MainMenu);
                }
                (false, false, true) => {
                    app_exit_events.send(AppExit::Success);
                }
                (return_button, main_menu_button, quit_button) => {
                    panic!("{return_button:?} {main_menu_button:?} {quit_button:?}");
                }
            }
        }
    }
}
