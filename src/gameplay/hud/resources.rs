use crate::prelude::{Fonts, PANEL_COLOR, SOFT_TEXT_COLOR};
use bevy::{
    ecs::system::Resource,
    prelude::{
        Commands, Font, Handle, NodeBundle, PositionType, Res, Style, TextStyle, UiRect, Val,
    },
};

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn create_hud_defaults(mut commands: Commands, fonts: Res<Fonts>) {
    commands.insert_resource(HudDefaults::new(fonts.default()));
}

#[derive(Resource)]
pub(crate) struct HudDefaults {
    pub(crate) text_style: TextStyle,
    pub(crate) background: NodeBundle,
}

impl HudDefaults {
    pub(crate) fn new(font: Handle<Font>) -> Self {
        Self {
            text_style: TextStyle {
                font,
                font_size: 16.0,
                color: SOFT_TEXT_COLOR,
            },
            background: NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    padding: UiRect::all(Val::Px(5.0)),
                    ..Style::default()
                },
                background_color: PANEL_COLOR.into(),
                ..NodeBundle::default()
            },
        }
    }
}
