use super::components::{MainMenuButton, ReturnButton};
use crate::prelude::{
    ApplicationState, Ctrl, Fonts, GameplayScreenState, InputChange, Key, Keys, QuitButton,
    StateBound, BAD_TEXT_COLOR, DEFAULT_TEXT_COLOR, GOOD_TEXT_COLOR, LARGE_FONT_SIZE,
    MEDIUM_SPACING,
};
use bevy::{app::AppExit, prelude::*};

#[allow(clippy::needless_pass_by_value)]
pub(super) fn spawn_menu(mut commands: Commands, fonts: Res<Fonts>) {
    let button = ButtonBundle {
        style: Style {
            width: Val::Px(250.0),
            height: Val::Px(70.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    };
    let font = fonts.default();

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
                ..default()
            },
            StateBound::<GameplayScreenState>::default(),
        ))
        .with_children(|parent| {
            parent
                .spawn((button.clone(), ReturnButton))
                .with_children(|parent| add_text(parent, &font, "Return", GOOD_TEXT_COLOR));
            parent
                .spawn((button.clone(), MainMenuButton))
                .with_children(|parent| add_text(parent, &font, "Main Menu", DEFAULT_TEXT_COLOR));
            parent
                .spawn((button, QuitButton))
                .with_children(|parent| add_text(parent, &font, "Quit", BAD_TEXT_COLOR));
        });
}

fn add_text(parent: &mut ChildBuilder, font: &Handle<Font>, text: &str, color: Color) {
    parent.spawn(TextBundle::from_section(
        text,
        TextStyle {
            font: font.clone(),
            font_size: LARGE_FONT_SIZE,
            color,
        },
    ));
}

#[allow(clippy::needless_pass_by_value)]
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
                    next_gameplay_state.set(GameplayScreenState::Inapplicable);
                    next_application_state.set(ApplicationState::MainMenu);
                }
                (false, false, true) => {
                    app_exit_events.send(AppExit);
                }
                (return_button, main_menu_button, quit_button) => {
                    panic!("{return_button:?} {main_menu_button:?} {quit_button:?}");
                }
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn manage_menu_keyboard_input(
    mut keys: Keys,
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
) {
    for combo in keys
        .combos(Ctrl::Without)
        .filter(|combo| combo.change == InputChange::JustPressed)
    {
        match combo.key {
            Key::Code(KeyCode::Escape) => {
                next_gameplay_state.set(GameplayScreenState::Base);
            }
            Key::Character('m') => {
                next_gameplay_state.set(GameplayScreenState::Inapplicable);
                next_application_state.set(ApplicationState::MainMenu);
            }
            _ => {}
        }
    }
}
