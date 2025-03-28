use crate::{application::ApplicationState, gameplay::GameplayScreenState};
use bevy::ecs::system::SystemId;
use bevy::prelude::{
    AlignItems, Commands, FlexDirection, In, JustifyContent, KeyCode, Local, NextState, Node, Res,
    ResMut, StateScoped, Text, UiRect, Val, World,
};
use hud::{BAD_TEXT_COLOR, ButtonBuilder, Fonts, PANEL_COLOR, SMALL_SPACING, WARN_TEXT_COLOR};
use keyboard::KeyBindings;
use manual::ManualSection;
use std::time::Instant;
use util::log_if_slow;

#[derive(Debug)]
pub(super) struct MainMenuSystem(SystemId<(), ()>);

#[allow(clippy::needless_pass_by_value)]
pub(super) fn create_main_menu_system(world: &mut World) -> MainMenuSystem {
    MainMenuSystem(world.register_system_cached(to_main_menu))
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn spawn_death_screen(
    In(main_menu_system): In<MainMenuSystem>,
    mut commands: Commands,
    fonts: Res<Fonts>,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Node::default()
            },
            StateScoped(GameplayScreenState::Death),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(200.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(SMALL_SPACING),
                        ..Node::default()
                    },
                    PANEL_COLOR,
                ))
                .with_children(|parent| {
                    parent
                        .spawn(Node {
                            width: Val::Px(250.0),
                            height: Val::Px(70.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..Node::default()
                        })
                        .with_children(|parent| {
                            parent.spawn((Text::from("You died"), BAD_TEXT_COLOR, fonts.largish()));
                        });

                    ButtonBuilder::new(
                        "Main menu",
                        WARN_TEXT_COLOR,
                        fonts.regular(),
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

    bindings.spawn(world, GameplayScreenState::Death, |bindings| {
        bindings.add(KeyCode::Escape, to_main_menu);
        bindings.add(KeyCode::Enter, to_main_menu);
        bindings.add(KeyCode::Space, to_main_menu);
    });

    world.spawn((
        ManualSection::new(&[("to main menu", "esc/enter/space")], 100),
        StateScoped(GameplayScreenState::Death),
    ));

    log_if_slow("create_death_screen_key_bindings", start);
}

fn to_main_menu(mut next_application_state: ResMut<NextState<ApplicationState>>) {
    next_application_state.set(ApplicationState::MainMenu);
}
