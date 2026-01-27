use crate::panel::{SCREEN_MARGINS, SMALL_PADDING};
use crate::{PANEL_COLOR, scroll_panel_with_content_entity};
use bevy::prelude::{
    AlignItems, Children, Commands, DespawnOnExit, Display, Entity, JustifyItems, Node, Pickable,
    SpawnRelated as _, States, Val, children,
};

pub fn spawn_modal_panel(commands: &mut Commands, state: impl States, width: Val) -> Entity {
    let (main_panel, content_entity) = scroll_panel_with_content_entity(commands);

    commands.spawn((
        // Entire screen
        Node {
            display: Display::Grid,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_items: JustifyItems::Center,
            ..Node::default()
        },
        DespawnOnExit(state),
        Pickable::IGNORE,
        children![(
            // Modal
            Node {
                margin: SCREEN_MARGINS,
                width,
                padding: SMALL_PADDING,
                ..Node::default()
            },
            PANEL_COLOR,
            Pickable::IGNORE,
            Children::spawn(main_panel)
        )],
    ));

    content_entity
}
