use crate::{
    DEFAULT_BUTTON_COLOR, DEFAULT_SCROLLBAR_COLOR, Fonts, HOVERED_BUTTON_COLOR,
    HOVERED_SCROLLBAR_COLOR, RunButton, SelectionList,
};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::{HoverMap, Hovered};
use bevy::prelude::{
    BackgroundColor, Button, ButtonInput, Changed, ChildOf, Commands, ComputedNode, Entity, In,
    Interaction, KeyCode, MessageReader, Or, Query, Res, ScrollPosition, SystemInput,
    UiGlobalTransform, UiScale, Vec2, Visibility, With, World, error,
};
use bevy::ui_widgets::{CoreScrollbarDragState, CoreScrollbarThumb, Scrollbar};
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
    mut scrolled_node_query: Query<(&mut ScrollPosition, &ComputedNode)>,
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
                if let Ok((mut scroll_position, node)) = scrolled_node_query.get_mut(*entity) {
                    let max_scroll = max_scroll(node);
                    scroll_position.x = (scroll_position.x - dx).clamp(0.0, max_scroll.x);
                    scroll_position.y = (scroll_position.y - dy).clamp(0.0, max_scroll.y);
                }
            }
        }
    }
}

fn max_scroll(node: &ComputedNode) -> Vec2 {
    Vec2::new(
        (node.content_size.x - node.size.x).max(0.0) * node.inverse_scale_factor,
        (node.content_size.y - node.size.y).max(0.0) * node.inverse_scale_factor,
    )
}

pub(crate) fn toggle_scroll_bar(
    mut bars: Query<(&mut Visibility, &Scrollbar)>,
    computed_nodes: Query<&ComputedNode>,
) {
    for (mut visibility, scrollbar) in &mut bars {
        let Ok(computed_node) = computed_nodes.get(scrollbar.target) else {
            error!("Computed node of scroll bar not found");
            continue;
        };

        *visibility = if max_scroll(computed_node).y == 0.0 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

pub(crate) fn update_scroll_thumb(
    mut q_thumb: Query<
        (&mut BackgroundColor, &Hovered, &CoreScrollbarDragState),
        (
            With<CoreScrollbarThumb>,
            Or<(Changed<Hovered>, Changed<CoreScrollbarDragState>)>,
        ),
    >,
) {
    for (mut thumb_bg, Hovered(is_hovering), drag) in &mut q_thumb {
        let color = if *is_hovering || drag.dragging {
            HOVERED_SCROLLBAR_COLOR
        } else {
            DEFAULT_SCROLLBAR_COLOR
        };

        if *thumb_bg != color {
            *thumb_bg = color;
        }
    }
}

#[expect(clippy::needless_pass_by_value)]
pub fn scroll_to_selection(
    In(selection_list): In<Entity>,
    ui_scale: Res<UiScale>,
    mut scroll_lists: Query<(&SelectionList, &ComputedNode, &mut ScrollPosition)>,
    parents: Query<&ChildOf>,
    global_transforms: Query<&UiGlobalTransform>,
) {
    // The middle of the list is also the middle of the screen. This makes using `GlobalTransform` convenient.

    let (selection_list, selection_node, mut scroll_position) = scroll_lists
        .get_mut(selection_list)
        .expect("The selection list entity should be found");

    if let Some(selected) = selection_list.selected {
        let transation = global_transforms
            .get(selected)
            .expect("Selected item should have a global transform")
            .translation;

        let parent = parents
            .get(selected)
            .expect("Selected item should have a parent")
            .parent();
        let parent_transation = global_transforms
            .get(parent)
            .expect("Parent of selected item should have a global transform")
            .translation;

        let max_scroll = max_scroll(selection_node);
        scroll_position.y = (scroll_position.y + (transation.y - parent_transation.y) / ui_scale.0)
            .clamp(0.0, max_scroll.y);
    }
}
