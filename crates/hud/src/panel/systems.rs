use crate::{DEFAULT_SCROLLBAR_COLOR, HOVERED_SCROLLBAR_COLOR, panel::max_scroll};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::{HoverMap, Hovered};
use bevy::prelude::{
    BackgroundColor, ButtonInput, Changed, Commands, ComputedNode, Hsla, Insert, KeyCode,
    MessageReader, Node, On, Or, Query, Res, ScrollPosition, Val, Visibility, With, error,
};
use bevy::ui::Outline;
use bevy::ui_widgets::{CoreScrollbarDragState, CoreScrollbarThumb, Scrollbar};
use fastrand::f32 as rand_f32;
use std::mem::swap;

/// Updates the scroll position of scrollable nodes in response to mouse input
#[expect(clippy::needless_pass_by_value)]
pub(super) fn update_scroll_position(
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

pub(super) fn toggle_scroll_bar_visibility(
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

pub(super) fn update_scroll_thumb_color(
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
pub(crate) fn add_node_outlines(trigger: On<Insert, Node>, mut commands: Commands) {
    commands.entity(trigger.entity).insert(Outline {
        width: Val::Px(2.0),
        offset: Val::ZERO,
        color: Hsla {
            hue: 360.0 * rand_f32(),
            saturation: 1.0,
            lightness: 0.7,
            alpha: 0.7,
        }
        .into(),
    });
}
