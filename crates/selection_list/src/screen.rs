use crate::SelectionList;
use bevy::prelude::{
    AlignItems, Children, Commands, Display, Entity, JustifyContent, Node, Pickable,
    RepeatedGridTrack, Spawn, SpawnRelated as _, States, Val,
};
use hud::{SMALL_SPACING, scroll_panel_with_content_entity, spawn_panel_root};

/// Returns the entities of the list node and the detail node
#[must_use]
pub fn selection_list_detail_screen<S: States>(
    commands: &mut Commands,
    state: S,
) -> (Entity, Entity) {
    let (list_panel, list_entity) = scroll_panel_with_content_entity(commands);
    commands
        .entity(list_entity)
        .insert(SelectionList::default());

    let (detail_panel, detail_entity) = scroll_panel_with_content_entity(commands);

    let content_panel = Spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::Grid,
            grid_template_columns: vec![
                RepeatedGridTrack::flex(1, 1.0),
                RepeatedGridTrack::flex(1, 3.0),
            ],
            grid_template_rows: vec![RepeatedGridTrack::auto(1)],
            column_gap: SMALL_SPACING,
            align_items: AlignItems::Start,
            justify_content: JustifyContent::Start,
            ..Node::default()
        },
        Pickable::IGNORE,
        Children::spawn((list_panel, detail_panel)),
    ));

    spawn_panel_root(commands, state, content_panel);

    (list_entity, detail_entity)
}
