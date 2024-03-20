use crate::prelude::{
    ApplicationState, Ctrl, Fonts, GameplayScreenState, InputChange, Key, Keys, StateBound,
    BAD_TEXT_COLOR, PANEL_COLOR, SMALL_SPACING, WARN_TEXT_COLOR,
};
use bevy::prelude::*;

#[allow(clippy::needless_pass_by_value)]
pub(super) fn spawn_death_screen(mut commands: Commands, fonts: Res<Fonts>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            StateBound::<GameplayScreenState>::default(),
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
                                fonts.largish(BAD_TEXT_COLOR),
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
                                fonts.large(WARN_TEXT_COLOR),
                            ));
                        });
                });
        });
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn manage_death_keyboard_input(
    mut keys: Keys,
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
) {
    for _ in keys.combos(Ctrl::Without).filter(|combo| {
        matches!(
            combo.key,
            Key::Code(KeyCode::Escape | KeyCode::Enter | KeyCode::F12 | KeyCode::Space)
        ) && combo.change == InputChange::JustPressed
    }) {
        next_application_state.set(ApplicationState::MainMenu);
        next_gameplay_state.set(GameplayScreenState::Inapplicable);
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn manage_death_button_input(
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
