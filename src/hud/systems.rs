use crate::hud::scrolling_list::ScrollingList;
use crate::hud::{
    DefaultPanel, Fonts, RunButton, RunButtonContext, DEFAULT_BUTTON_COLOR, HOVERED_BUTTON_COLOR,
};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::{
    BackgroundColor, Button, Changed, Commands, Entity, EventReader, In, Interaction, Node, Parent,
    Query, Style, With, Without, World,
};

pub(super) fn create_default_panel(world: &mut World) {
    world.insert_resource(DefaultPanel::new());
}

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
        color.0 = match *interaction {
            Interaction::Hovered | Interaction::Pressed => HOVERED_BUTTON_COLOR,
            Interaction::None => DEFAULT_BUTTON_COLOR,
        };
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_button_input<C: RunButtonContext>(
    mut commands: Commands,
    interaction_query: Query<(&Interaction, &RunButton<C>), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, button) in &interaction_query {
        if *interaction == Interaction::Pressed {
            button.run(&mut commands);
        }
    }
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn manage_scrolling_lists(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut scrolling_lists: Query<(&mut ScrollingList, &mut Style, &Parent, &Node, &Interaction)>,
    parent_nodes: Query<(&Node, &Style), Without<ScrollingList>>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (mut scrolling_list, mut style, parent, list_node, interaction) in &mut scrolling_lists
        {
            if interaction != &Interaction::None {
                let (parent_node, parent_style) = parent_nodes
                    .get(parent.get())
                    .expect("Parent node should be found");
                style.top =
                    scrolling_list.scroll(list_node, parent_node, parent_style, mouse_wheel_event);
            }
        }
    }
}

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn resize_scrolling_lists(
    mut scrolling_lists: Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    parent_nodes: Query<(&Node, &Style), Without<ScrollingList>>,
) {
    for (mut scrolling_list, mut style, parent, list_node) in &mut scrolling_lists {
        let (parent_node, parent_style) = parent_nodes
            .get(parent.get())
            .expect("Parent node should be found");
        style.top = scrolling_list.resize(list_node, parent_node, parent_style);
    }
}

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn trigger_button_action<C: RunButtonContext>(
    In(entity): In<Entity>,
    mut commands: Commands,
    run_buttons: Query<&RunButton<C>>,
) {
    run_buttons
        .get(entity)
        .expect("Triggered run button should be found")
        .run(&mut commands);
}
