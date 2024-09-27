use crate::hud::PANEL_COLOR;
use bevy::prelude::{NodeBundle, PositionType, Resource, Style, UiRect, Val};

#[derive(Resource)]
pub(crate) struct DefaultPanel(NodeBundle);

impl DefaultPanel {
    pub(super) fn new() -> Self {
        Self(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                padding: UiRect::all(Val::Px(5.0)),
                ..Style::default()
            },
            background_color: PANEL_COLOR.into(),
            ..NodeBundle::default()
        })
    }

    pub(crate) const fn ref_(&self) -> &NodeBundle {
        &self.0
    }

    pub(crate) fn cloned(&self) -> NodeBundle {
        self.0.clone()
    }
}
