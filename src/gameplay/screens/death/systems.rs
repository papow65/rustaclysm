use crate::common::{
    Fonts, Key, Keys, BAD_TEXT_COLOR, PANEL_COLOR, SMALL_SPACING, WARN_TEXT_COLOR,
};
use crate::{application::ApplicationState, gameplay::GameplayScreenState};
use bevy::prelude::{
    AlignItems, BuildChildren, Button, ButtonBundle, Changed, Commands, FlexDirection, Interaction,
    JustifyContent, KeyCode, NextState, NodeBundle, Query, Res, ResMut, StateScoped, Style,
    TextBundle, UiRect, Val, With,
};

#[expect(clippy::needless_pass_by_value)]
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

                    parent
                        .spawn(ButtonBundle {
                            style: Style {
                                width: Val::Px(250.0),
                                height: Val::Px(70.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..Style::default()
                            },
                            ..ButtonBundle::default()
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

#[expect(clippy::needless_pass_by_value)]
pub(super) fn manage_death_keyboard_input(
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    keys: Res<Keys>,
) {
    for _ in keys.just_pressed_without_ctrl().filter(|key| {
        matches!(
            **key,
            Key::Code(KeyCode::Escape | KeyCode::Enter | KeyCode::F12 | KeyCode::Space)
        )
    }) {
        next_application_state.set(ApplicationState::MainMenu);
    }
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn manage_death_button_input(
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    interactions: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    for &interaction in &interactions {
        if interaction == Interaction::Pressed {
            next_application_state.set(ApplicationState::MainMenu);
        }
    }
}
