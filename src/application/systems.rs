use crate::prelude::*;
use bevy::{app::AppExit, prelude::*};

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
    mut ui_scale: ResMut<UiScale>,
) {
    for combo in keys.combos(Ctrl::With) {
        match combo.key {
            Key::Character('c' | 'q') => {
                app_exit_events.send(AppExit);
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
pub(crate) fn despawn<T: States>(
    mut commands: Commands,
    entities: Query<Entity, With<StateBound<T>>>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
