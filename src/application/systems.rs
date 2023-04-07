use crate::prelude::*;
use bevy::prelude::*;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn maximize_window(mut windows: Query<&mut Window>) {
    for mut window in windows.iter_mut() {
        window.set_maximized(true);
    }
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn load_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Fonts::new(&asset_server));
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_button_hover(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Hovered => {
                *color = HOVERED_BUTTON_COLOR.into();
            }
            Interaction::None => {
                *color = DEFAULT_BUTTON_COLOR.into();
            }
            Interaction::Clicked => {}
        }
    }
}
