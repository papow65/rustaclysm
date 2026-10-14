use crate::{DEFAULT_BUTTON_COLOR, HOVERED_BUTTON_COLOR, RunButton};
use bevy::ecs::query::Spawned;
use bevy::picking::hover::Hovered;
use bevy::prelude::{
    Added, BackgroundColor, Changed, Commands, Entity, In, Query, SystemInput, With,
};
use bevy::ui::Pressed;
use bevy::ui_widgets::Button;
use std::fmt;

pub(super) fn manage_new_button(
    mut commands: Commands,
    mut interactions: Query<Entity, (Spawned, With<Button>)>,
) {
    for button_entity in &mut interactions {
        commands
            .entity(button_entity)
            .insert((Hovered::default(), DEFAULT_BUTTON_COLOR));
    }
}

pub(super) fn manage_interacted_button_color(
    mut interactions: Query<(&Hovered, &mut BackgroundColor), (Changed<Hovered>, With<Button>)>,
) {
    for (hovered, mut color) in &mut interactions {
        *color = if hovered.get() {
            HOVERED_BUTTON_COLOR
        } else {
            DEFAULT_BUTTON_COLOR
        };
    }
}

/// Button presses
pub fn manage_button_input<I: fmt::Debug + SystemInput + 'static>(
    mut commands: Commands,
    interactions: Query<&RunButton<I>, (Added<Pressed>, With<Button>)>,
) where
    <I as SystemInput>::Inner<'static>: Clone + fmt::Debug + Send + Sync,
{
    for button in &interactions {
        button.run(&mut commands);
    }
}

/// Mimic a button press
pub fn trigger_button_action<I: fmt::Debug + SystemInput + 'static>(
    In(entity): In<Entity>,
    mut commands: Commands,
    run_buttons: Query<&RunButton<I>>,
) where
    <I as SystemInput>::Inner<'static>: Clone + fmt::Debug + Send + Sync,
{
    run_buttons
        .get(entity)
        .expect("Triggered run button should be found")
        .run(&mut commands);
}
