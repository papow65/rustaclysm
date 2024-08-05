use crate::common::{Fonts, PANEL_COLOR, SOFT_TEXT_COLOR};
use bevy::{
    ecs::system::Resource,
    prelude::{
        Commands, NodeBundle, PositionType, Res, Style, TextSection, TextStyle, UiRect, Val,
    },
};

#[allow(clippy::needless_pass_by_value)]
pub(super) fn spawn_hud_resources(mut commands: Commands, fonts: Res<Fonts>) {
    commands.insert_resource(HudDefaults::new(&fonts));
    commands.insert_resource(StatusTextSections::default());
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn despawn_hud_resources(mut commands: Commands) {
    commands.remove_resource::<HudDefaults>();
    commands.remove_resource::<StatusTextSections>();
}

#[derive(Resource)]
pub(super) struct HudDefaults {
    pub(super) text_style: TextStyle,
    pub(super) background: NodeBundle,
}

impl HudDefaults {
    pub(super) fn new(fonts: &Fonts) -> Self {
        Self {
            text_style: fonts.regular(SOFT_TEXT_COLOR),
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

#[derive(Debug, Default, Resource)]
pub(super) struct StatusTextSections {
    pub(super) fps: TextSection,
    pub(super) time: TextSection,
    pub(super) health: [TextSection; 2],
    pub(super) stamina: [TextSection; 2],
    pub(super) speed: [TextSection; 3],
    pub(super) player_action_state: TextSection,
    pub(super) wielded: Vec<TextSection>,
    pub(super) enemies: Vec<TextSection>,
    pub(super) details: Vec<TextSection>,
}
