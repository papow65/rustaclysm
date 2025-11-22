use crate::SelectionListStep;
use bevy::platform::hash::RandomState;
use bevy::prelude::{Component, Entity};
use indexmap::IndexSet;

/// Used on a selection list item, for the selection list that contains the item.
///
/// Use `commands.entity(list_entity)
///    .add_one_related::<SelectableItemIn>(item_entity)` to add an item in a selection list.
///
/// Somewhat similar to `ChildOf`
#[derive(Clone, Copy, Debug, Component)]
#[relationship(relationship_target = SelectionListItems)]
pub struct SelectableItemIn {
    pub(crate) selection_list: Entity,
}

/// Used on a selection list, for all items in that selection list.
///
/// Somewhat similar to `Children`
#[derive(Debug, Component)]
#[relationship_target(relationship = SelectableItemIn)]
pub struct SelectionListItems {
    items: IndexSet<Entity, RandomState>,
}

impl SelectionListItems {
    #[must_use]
    pub(crate) fn first(&self) -> Option<Entity> {
        self.items.iter().next().copied()
    }

    #[must_use]
    pub fn offset(&self, reference_child: Entity, step: SelectionListStep) -> Entity {
        let previous_index = self
            .items
            .get_index_of(&reference_child)
            .expect("Selected item in selection list should be found");
        let max_index = self.items.len() - 1;
        let new_index = if step.is_backwards() {
            if previous_index == 0 {
                // Loop to the bottom
                max_index
            } else {
                previous_index - step.amount().min(previous_index)
            }
        } else if previous_index == max_index {
            // Loop to the top
            0
        } else {
            (previous_index + step.amount()).min(max_index)
        };
        *self
            .items
            .get_index(new_index)
            .expect("An item should exist in selection list atr the given position")
    }
}

/// Used on a selection list item, for the selection list by which it is selected.
///
/// Use `commands.entity(list_entity)
///    .add_one_related::<SelectedItemIn>(item_entity)` to select an item in a selection list.
#[derive(Clone, Copy, Debug, Component)]
#[relationship(relationship_target = SelectedItemOf)]
pub struct SelectedItemIn {
    pub(crate) selection_list: Entity,
}

/// Used on a selection list, for the selected items in that selection list.
#[derive(Debug, Component)]
#[relationship_target(relationship=SelectedItemIn)]
pub struct SelectedItemOf {
    selected: Entity,
}

impl SelectedItemOf {
    #[must_use]
    pub const fn selected(&self) -> Entity {
        self.selected
    }
}
