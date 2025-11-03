use crate::LoadingIndicatorState;
use bevy::prelude::{
    AlignItems, App, AppExtStates as _, Commands, Component, DespawnOnExit, FixedUpdate,
    GlobalZIndex, IntoScheduleConfigs as _, JustifyContent, Node, OnEnter, Plugin, PositionType,
    Res, Single, Text, Val, With, in_state,
};
use hud::{DEFAULT_BUTTON_COLOR, Fonts, HARD_TEXT_COLOR};
use util::log_transition_plugin;

#[derive(Component)]
#[component(immutable)]
struct LoadingText;

pub struct LoadingIndicatorPlugin;

impl Plugin for LoadingIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_computed_state::<LoadingIndicatorState>();
        app.add_plugins(log_transition_plugin::<LoadingIndicatorState>);

        app.add_systems(OnEnter(LoadingIndicatorState), spawn_loading);
        app.add_systems(FixedUpdate, animate.run_if(in_state(LoadingIndicatorState)));
    }
}

#[expect(clippy::needless_pass_by_value)]
fn spawn_loading(mut commands: Commands, fonts: Res<Fonts>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..Node::default()
            },
            GlobalZIndex(3),
            DespawnOnExit(LoadingIndicatorState),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(70.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Node::default()
                    },
                    DEFAULT_BUTTON_COLOR,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        LoadingText,
                        Text::from("Loading..."),
                        HARD_TEXT_COLOR,
                        fonts.large(),
                    ));
                });
        });
}

fn animate(mut text: Single<&mut Text, With<LoadingText>>) {
    let now: &str = text.0.as_str();
    text.0 = String::from(match now {
        "Loading..." => "Loading ..",
        "Loading .." => "Loading. .",
        "Loading. ." => "Loading.. ",
        _ => "Loading...",
    });
}
