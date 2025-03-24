use crate::loading::LoadingIndicatorState;
use bevy::prelude::{
    AlignItems, App, AppExtStates as _, BuildChildren as _, ChildBuild as _, Commands,
    GlobalZIndex, JustifyContent, Node, OnEnter, Plugin, PositionType, Res, StateScoped, Text, Val,
};
use hud::{DEFAULT_BUTTON_COLOR, Fonts, HARD_TEXT_COLOR};
use util::log_transition_plugin;

pub(crate) struct LoadingIndicatorPlugin;

impl Plugin for LoadingIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_computed_state::<LoadingIndicatorState>();
        app.enable_state_scoped_entities::<LoadingIndicatorState>();
        app.add_plugins(log_transition_plugin::<LoadingIndicatorState>);

        app.add_systems(OnEnter(LoadingIndicatorState), spawn_loading);
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
            StateScoped(LoadingIndicatorState),
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
                    parent.spawn((Text::from("Loading..."), HARD_TEXT_COLOR, fonts.large()));
                });
        });
}
