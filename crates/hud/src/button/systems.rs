use crate::{DEFAULT_BUTTON_COLOR, HOVERED_BUTTON_COLOR, RunButton};
use bevy::prelude::{
    BackgroundColor, Button, Changed, Commands, Entity, In, Interaction, Query, SystemInput, With,
};
use std::fmt;

pub(super) fn manage_button_color(
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interactions {
        *color = match *interaction {
            Interaction::Hovered | Interaction::Pressed => HOVERED_BUTTON_COLOR,
            Interaction::None => DEFAULT_BUTTON_COLOR,
        };
    }
}

/// Button presses
pub fn manage_button_input<I: fmt::Debug + SystemInput + 'static>(
    mut commands: Commands,
    interactions: Query<(&Interaction, &RunButton<I>), (Changed<Interaction>, With<Button>)>,
) where
    <I as SystemInput>::Inner<'static>: Clone + fmt::Debug + Send + Sync,
{
    for (interaction, button) in &interactions {
        if *interaction == Interaction::Pressed {
            button.run(&mut commands);
        }
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
