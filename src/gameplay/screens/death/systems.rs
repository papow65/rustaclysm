use crate::prelude::*;
use bevy::{input::ButtonState, prelude::*};

const SPACING: f32 = 5.0;
const FONT_SIZE: f32 = 16.0;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_death_screen(mut commands: Commands, fonts: Res<Fonts>) {
    let font = fonts.default();

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .insert(StateBound::<GameplayScreenState>::default())
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(300.0),
                        height: Val::Px(200.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Px(SPACING)),
                        ..default()
                    },
                    background_color: PANEL_COLOR.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Px(250.0),
                                height: Val::Px(70.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "You died",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: FONT_SIZE,
                                    color: DEFAULT_TEXT_COLOR,
                                },
                            ));
                        });

                    parent
                        .spawn(ButtonBundle {
                            style: Style {
                                width: Val::Px(250.0),
                                height: Val::Px(70.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "Main menu",
                                TextStyle {
                                    font,
                                    font_size: FONT_SIZE,
                                    color: BAD_TEXT_COLOR,
                                },
                            ));
                        });
                });
        });
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_death_keyboard_input(
    mut keys: Keys,
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
) {
    for (state, combo) in keys.combos() {
        if state != ButtonState::Pressed {
            continue;
        }

        if let KeyCombo::KeyCode(Ctrl::Without, KeyCode::Escape | KeyCode::Return | KeyCode::F12) =
            combo
        {
            next_application_state.set(ApplicationState::MainMenu);
            next_gameplay_state.set(GameplayScreenState::Inapplicable);
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_death_button_input(
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    interactions: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    for &interaction in &interactions {
        if interaction == Interaction::Pressed {
            next_application_state.set(ApplicationState::MainMenu);
            next_gameplay_state.set(GameplayScreenState::Inapplicable);
        }
    }
}
