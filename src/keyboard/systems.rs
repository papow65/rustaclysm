use crate::keyboard::{Ctrl, Held, KeyBindings, key_binding::KeyBinding, keys::Keys};
use crate::manual::ManualSection;
use bevy::app::AppExit;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::{
    ButtonInput, Commands, Entity, EventReader, Events, In, IntoSystem as _, KeyCode, Query, Res,
    ResMut, UiScale, World, debug,
};

enum ZoomUiDirection {
    In,
    Out,
}

pub(super) fn create_global_key_bindings(world: &mut World) {
    KeyBindings::<_, Ctrl, ()>::spawn_global(
        world,
        |bindings| {
            bindings.add('+', (|| ZoomUiDirection::In).pipe(zoom_ui));
            bindings.add('-', (|| ZoomUiDirection::Out).pipe(zoom_ui));
            bindings.add('q', quit);
            if !cfg!(windows) {
                bindings.add('c', quit);
            }
        },
        ManualSection::new(
            &[
                ("zoom ui", "ctrl +/-"),
                ("quit", if cfg!(windows) { "ctrl q" } else { "ctrl c/q" }),
            ],
            u8::MAX,
        ),
    );
}

fn zoom_ui(In(direction): In<ZoomUiDirection>, mut ui_scale: ResMut<UiScale>) {
    let zoom = match direction {
        ZoomUiDirection::In => 1,
        ZoomUiDirection::Out => -1,
    };
    let px = zoom + (16.0 * ui_scale.0) as i8;
    let px = px.clamp(4, 64);
    ui_scale.0 = f32::from(px) / 16.0;
    debug!("UI scale: {ui_scale:?}");
}

fn quit(mut app_exit_events: ResMut<Events<AppExit>>) {
    app_exit_events.send(AppExit::Success);
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn preprocess_keyboard_input(
    mut keyboard_inputs: EventReader<KeyboardInput>,
    key_states: Res<ButtonInput<KeyCode>>,
) -> Keys {
    Keys::new(&mut keyboard_inputs, &key_states)
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn manage_binded_keyboard_input(
    In(keys): In<Keys>,
    mut commands: Commands,
    key_bindings_fresh_without_ctrl: Query<(Entity, &KeyBinding<(), ()>)>,
    key_bindings_held_without_ctrl: Query<(Entity, &KeyBinding<(), Held>)>,
    key_bindings_fresh_with_ctrl: Query<(Entity, &KeyBinding<Ctrl, ()>)>,
    key_bindings_held_with_ctrl: Query<(Entity, &KeyBinding<Ctrl, Held>)>,
) {
    keys.process(
        &mut commands,
        &key_bindings_fresh_without_ctrl,
        &key_bindings_held_without_ctrl,
        &key_bindings_fresh_with_ctrl,
        &key_bindings_held_with_ctrl,
    );
}
