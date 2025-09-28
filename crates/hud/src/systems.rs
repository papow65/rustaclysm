use crate::{DEFAULT_BUTTON_COLOR, Fonts, HOVERED_BUTTON_COLOR, RunButton, SelectionList};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::prelude::{
    BackgroundColor, Button, ButtonInput, Changed, Commands, Entity, In, Interaction, KeyCode,
    MessageReader, Query, Res, ScrollPosition, SystemInput, UiScale, UiTransform, Val, With, World,
    error,
};
use std::{fmt, mem::swap};

pub(super) fn load_fonts(world: &mut World) {
    let asset_server = world.get_resource().expect("AssetServer should exist");
    let fonts = Fonts::new(asset_server);
    // Using 'commands.insert_resource' in bevy 0.14-rc2 doesn't work properly.
    world.insert_resource(fonts);
}

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

/// Updates the scroll position of scrollable nodes in response to mouse input
#[expect(clippy::needless_pass_by_value)]
pub(crate) fn update_scroll_position(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut scrolled_node_query: Query<&mut ScrollPosition>,
) {
    const LINE_HEIGHT: f32 = 20.0;

    for mouse_wheel_event in mouse_wheel_events.read() {
        let (mut dx, mut dy) = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => (
                mouse_wheel_event.x * LINE_HEIGHT,
                mouse_wheel_event.y * LINE_HEIGHT,
            ),
            MouseScrollUnit::Pixel => (mouse_wheel_event.x, mouse_wheel_event.y),
        };

        if keyboard_input.pressed(KeyCode::ControlLeft)
            || keyboard_input.pressed(KeyCode::ControlRight)
        {
            swap(&mut dx, &mut dy);
        }

        for hit_data in hover_map.values() {
            for entity in hit_data.keys() {
                if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
                    scroll_position.x -= dx;
                    scroll_position.y -= dy;
                }
            }
        }
    }
}

#[expect(clippy::needless_pass_by_value)]
pub fn scroll_to_selection(
    In(selection_list): In<Entity>,
    ui_scale: Res<UiScale>,
    mut scroll_lists: Query<(&SelectionList, &mut ScrollPosition)>,
    transforms: Query<&UiTransform>,
) {
    let (selection_list, mut scroll_position) = scroll_lists
        .get_mut(selection_list)
        .expect("The selection list entity should be found");

    if let Some(selected) = selection_list.selected {
        let selected_transform = transforms
            .get(selected)
            .expect("Selected item should be found");

        let Val::Px(y_transation) = selected_transform.translation.y else {
            error!(
                "Expected pixel value for vertical translation, but got {:?}",
                selected_transform.translation.y
            );

            return;
        };

        // The translation is already offset by the scroll position, so we can use this:
        scroll_position.y += y_transation / ui_scale.0;
    }
}
