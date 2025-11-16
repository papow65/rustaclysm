use crate::{SelectionList, panel::max_scroll};
use bevy::prelude::{
    ChildOf, ComputedNode, Entity, In, Query, Res, ScrollPosition, UiGlobalTransform, UiScale,
};

#[expect(clippy::needless_pass_by_value)]
pub fn scroll_to_selection(
    In(selection_list): In<Entity>,
    ui_scale: Res<UiScale>,
    mut scroll_lists: Query<(&SelectionList, &ComputedNode, &mut ScrollPosition)>,
    parents: Query<&ChildOf>,
    global_transforms: Query<&UiGlobalTransform>,
) {
    // The middle of the list is also the middle of the screen. This makes using `GlobalTransform` convenient.

    let (selection_list, selection_node, mut scroll_position) = scroll_lists
        .get_mut(selection_list)
        .expect("The selection list entity should be found");

    if let Some(selected) = selection_list.selected {
        let transation = global_transforms
            .get(selected)
            .expect("Selected item should have a global transform")
            .translation;

        let parent = parents
            .get(selected)
            .expect("Selected item should have a parent")
            .parent();
        let parent_transation = global_transforms
            .get(parent)
            .expect("Parent of selected item should have a global transform")
            .translation;

        let max_scroll = max_scroll(selection_node);
        scroll_position.y = (scroll_position.y + (transation.y - parent_transation.y) / ui_scale.0)
            .clamp(0.0, max_scroll.y);
    }
}
