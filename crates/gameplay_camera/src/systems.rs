use crate::{CameraDirection, CameraZoom, ZoomDirection, ZoomDistance};
use bevy::ecs::{schedule::ScheduleConfigs, system::ScheduleSystem};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::{
    ButtonInput, Camera3d, In, IntoScheduleConfigs as _, MessageReader, MouseButton, Node, Query,
    Res, ResMut, Single, SystemCondition as _, SystemSet, Transform, Vec3, With, on_message,
    resource_exists_and_changed,
};
use bevy::{camera::visibility::RenderLayers, picking::hover::HoverMap};
use std::time::Instant;
use util::log_if_slow;

#[derive(Clone, Debug, PartialEq, Eq, Hash, SystemSet)]
pub struct UpdateCameraOffset;

pub fn manage_camera_offset() -> ScheduleConfigs<ScheduleSystem> {
    (
        (
            manage_camera_zoom.run_if(on_message::<MouseWheel>),
            manage_camera_direction.run_if(on_message::<MouseMotion>),
        ),
        update_camera_offset
            .run_if(
                resource_exists_and_changed::<CameraDirection>
                    .or_else(resource_exists_and_changed::<CameraZoom>),
            )
            .in_set(UpdateCameraOffset),
    )
        .into_configs()
}

pub fn toggle_map(
    In(zoom_distance): In<ZoomDistance>,
    mut camera_zoom: ResMut<CameraZoom>,
    mut camera_layers: Single<&mut RenderLayers, With<Camera3d>>,
) {
    let start = Instant::now();

    **camera_layers = if showing_map(&camera_layers) {
        camera_zoom.zoom_to_tiles(zoom_distance);
        (*camera_layers).clone().with(1).without(2)
    } else {
        camera_zoom.zoom_to_map(zoom_distance);
        (*camera_layers).clone().without(1).with(2)
    };

    log_if_slow("toggle_map", start);
}

fn zoom(
    camera_zoom: &mut CameraZoom,
    camera_layers: &mut Single<&mut RenderLayers, With<Camera3d>>,
    direction: ZoomDirection,
) {
    camera_zoom.zoom(direction);

    if showing_map(camera_layers) {
        if camera_zoom.zoom_tiles_only() {
            ***camera_layers = camera_layers.clone().with(1).without(2);
        }
    } else if camera_zoom.zoom_map_only() {
        ***camera_layers = camera_layers.clone().without(1).with(2);
    }
}

fn showing_map(camera_layers: &RenderLayers) -> bool {
    camera_layers.intersects(&RenderLayers::layer(2))
}

pub fn reset_camera_angle(mut camera_direction: ResMut<CameraDirection>) {
    let start = Instant::now();

    camera_direction.reset_angle();

    log_if_slow("reset_camera_angle", start);
}

#[expect(clippy::needless_pass_by_value)]
fn manage_camera_zoom(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut camera_zoom: ResMut<CameraZoom>,
    mut camera_layers: Single<&mut RenderLayers, With<Camera3d>>,
    ui_nodes: Query<(), With<Node>>,
) {
    let start = Instant::now();

    let hovered_ui_node = hover_map
        .values()
        .flat_map(|hit_data| hit_data.keys())
        .find(|entity| ui_nodes.contains(**entity));
    if hovered_ui_node.is_some() {
        return;
    }

    for scroll_event in &mut mouse_wheel_events.read() {
        zoom(
            &mut camera_zoom,
            &mut camera_layers,
            if 0.0 < scroll_event.y {
                ZoomDirection::In
            } else {
                ZoomDirection::Out
            },
        );
    }

    log_if_slow("manage_mouse_scroll_input", start);
}

#[expect(clippy::needless_pass_by_value)]
fn manage_camera_direction(
    mut mouse_motion_messages: MessageReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut camera_direction: ResMut<CameraDirection>,
) {
    let start = Instant::now();

    if mouse_buttons.pressed(MouseButton::Middle) {
        let delta_sum = mouse_motion_messages
            .read()
            .map(|motion_message| motion_message.delta)
            .sum();
        camera_direction.adjust_angle(delta_sum);
    }

    log_if_slow("manage_mouse_button_input", start);
}

pub fn manage_zoom(
    In(zoom_direction): In<ZoomDirection>,
    mut camera_zoom: ResMut<CameraZoom>,
    mut camera_layers: Single<&mut RenderLayers, With<Camera3d>>,
) {
    let start = Instant::now();

    zoom(&mut camera_zoom, &mut camera_layers, zoom_direction);

    log_if_slow("manage_zoom", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn update_camera_offset(
    camera_direction: Res<CameraDirection>,
    camera_zoom: Res<CameraZoom>,
    mut camera_transform: Single<&mut Transform, With<Camera3d>>,
) {
    let start = Instant::now();

    camera_transform.translation = camera_direction.direction() * camera_zoom.distance();
    camera_transform.look_at(Vec3::ZERO, Vec3::Y);

    log_if_slow("update_camera_offset", start);
}
