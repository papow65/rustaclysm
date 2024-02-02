use crate::prelude::*;
use bevy::{app::AppExit, input::ButtonState, prelude::*};

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn maximize_window(mut windows: Query<&mut Window>) {
    for mut window in &mut windows {
        window.set_maximized(true);
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn load_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Fonts::new(&asset_server));
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_button_hover(
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interactions {
        match *interaction {
            Interaction::Hovered => {
                *color = HOVERED_BUTTON_COLOR.into();
            }
            Interaction::None => {
                *color = DEFAULT_BUTTON_COLOR.into();
            }
            Interaction::Pressed => {}
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_global_keyboard_input(
    mut keys: Keys,
    mut app_exit_events: ResMut<Events<AppExit>>,
) {
    for (state, combo) in keys.combos() {
        if state != ButtonState::Pressed {
            continue;
        }

        if let KeyCombo::KeyCode(Ctrl::With, KeyCode::KeyC | KeyCode::KeyQ) = combo {
            app_exit_events.send(AppExit);
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn despawn<T: States>(
    mut commands: Commands,
    entities: Query<Entity, With<StateBound<T>>>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
