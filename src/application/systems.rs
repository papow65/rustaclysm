use crate::prelude::*;
use bevy::{
    app::AppExit,
    input::{keyboard::KeyboardInput, mouse::MouseWheel},
    prelude::*,
};

#[allow(clippy::needless_pass_by_value)]
pub(super) fn maximize_window(mut windows: Query<&mut Window>) {
    for mut window in &mut windows {
        window.set_maximized(true);
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn load_fonts(world: &mut World) {
    let asset_server = world.get_resource().expect("AssetServer should exist");
    let fonts = Fonts::new(asset_server);
    // Using 'commands.insert_resource' in bevy 0.14-rc2 doesn't work properly.
    world.insert_resource(fonts);
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn preprocess_keyboard_input(
    mut keyboard_inputs: EventReader<KeyboardInput>,
    key_states: Res<ButtonInput<KeyCode>>,
    mut keys: ResMut<Keys>,
) {
    keys.update(&mut keyboard_inputs, &key_states);
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn manage_button_color(
    mut interactions: Query<(&Interaction, &mut UiImage), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut image) in &mut interactions {
        image.color = match *interaction {
            Interaction::Hovered | Interaction::Pressed => HOVERED_BUTTON_COLOR,
            Interaction::None => DEFAULT_BUTTON_COLOR,
        };
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn manage_global_keyboard_input(
    keys: Res<Keys>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    mut ui_scale: ResMut<UiScale>,
) {
    for key_change in keys.with_ctrl() {
        match key_change.key {
            Key::Character('c' | 'q') => {
                app_exit_events.send(AppExit::Success);
            }
            Key::Character(resize @ ('+' | '-')) => {
                let px = if resize == '+' { 1 } else { -1 } + (16.0 * ui_scale.0) as i8;
                let px = px.clamp(4, 64);
                ui_scale.0 = f32::from(px) / 16.0;
                println!("UI scale: {ui_scale:?}");
            }
            _ => {}
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn manage_scrolling(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut scrolling_lists: Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
    parent_nodes: Query<(&Node, &Style), Without<ScrollingList>>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (mut scrolling_list, mut style, parent, list_node) in &mut scrolling_lists {
            let (parent_node, parent_style) = parent_nodes
                .get(parent.get())
                .expect("Parent node should be found");
            style.top =
                scrolling_list.scroll(list_node, parent_node, parent_style, mouse_wheel_event);
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
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
