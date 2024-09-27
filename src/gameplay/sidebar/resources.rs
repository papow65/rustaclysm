use bevy::prelude::{Commands, Resource, TextSection};

pub(super) fn spawn_sidebar_resources(mut commands: Commands) {
    commands.insert_resource(StatusTextSections::default());
}

pub(super) fn despawn_sidebar_resources(mut commands: Commands) {
    commands.remove_resource::<StatusTextSections>();
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
