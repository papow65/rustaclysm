use crate::hud::{DEFAULT_BUTTON_COLOR, Fonts, HOVERED_BUTTON_COLOR, RunButton, ScrollList};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::{
    BackgroundColor, Button, Changed, Commands, ComputedNode, Entity, EventReader, In, Interaction,
    Node, Parent, Query, SystemInput, With, Without, World,
};
use std::fmt;

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

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_button_input<I: SystemInput + 'static>(
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

#[expect(clippy::needless_pass_by_value)]
pub(super) fn manage_scroll_lists(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut scroll_lists: Query<(
        &mut ScrollList,
        &mut Node,
        &ComputedNode,
        &Parent,
        &Interaction,
    )>,
    parent_nodes: Query<(&Node, &ComputedNode), Without<ScrollList>>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (mut scroll_list, mut node, computed_node, parent, interaction) in &mut scroll_lists {
            if interaction != &Interaction::None {
                let (parent_node, parent_computed_node) = parent_nodes
                    .get(parent.get())
                    .expect("Parent node should be found");
                node.top = scroll_list.scroll(
                    computed_node,
                    parent_node,
                    parent_computed_node,
                    mouse_wheel_event,
                );
            }
        }
    }
}

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn resize_scroll_lists(
    mut scroll_lists: Query<(&mut ScrollList, &mut Node, &ComputedNode, &Parent)>,
    parent_nodes: Query<(&Node, &ComputedNode), Without<ScrollList>>,
) {
    for (mut scroll_list, mut style, computed_node, parent) in &mut scroll_lists {
        let (parent_node, parent_computed_node) = parent_nodes
            .get(parent.get())
            .expect("Parent node should be found");
        style.top = scroll_list.resize(computed_node, parent_node, parent_computed_node);
    }
}

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn trigger_button_action<I: SystemInput + 'static>(
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
