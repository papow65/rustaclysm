use crate::keyboard::{Ctrl, KeyBindings};
use crate::manual::ManualSection;
use bevy::app::AppExit;
use bevy::prelude::{Events, In, IntoSystem as _, ResMut, UiScale, World, debug};

enum ZoomUiDirection {
    In,
    Out,
}

pub(super) fn create_global_key_bindings(world: &mut World) {
    KeyBindings::<_, Ctrl, ()>::spawn_global(world, |bindings| {
        bindings.add('+', (|| ZoomUiDirection::In).pipe(zoom_ui));
        bindings.add('-', (|| ZoomUiDirection::Out).pipe(zoom_ui));
        bindings.add('q', quit);
        if !cfg!(windows) {
            bindings.add('c', quit);
        }
    });

    world.spawn(ManualSection::new(
        &[
            ("zoom ui", "ctrl +/-"),
            ("quit", if cfg!(windows) { "ctrl q" } else { "ctrl c/q" }),
        ],
        u8::MAX,
    ));
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
