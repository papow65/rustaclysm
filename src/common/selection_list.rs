use bevy::{
    ecs::entity::EntityHashMap,
    prelude::{Component, Entity},
};

#[derive(Component, Default)]
pub(crate) struct SelectionList {
    pub(crate) selected: Option<Entity>,
    first: Option<Entity>,
    last: Option<Entity>,
    previous_items: EntityHashMap<Entity>,
    next_items: EntityHashMap<Entity>,
}

impl SelectionList {
    pub(crate) fn append(&mut self, added: Entity) {
        if self.first.is_none() {
            self.first = Some(added);
            if self.selected.is_none() {
                self.selected = Some(added);
            }
        }

        if let Some(previous_item) = self.last {
            let Some(first_item) = self.first else {
                panic!("If there is a last item, there also should be a first item.")
            };
            self.previous_items.insert(added, previous_item);
            self.previous_items.insert(first_item, added);
            self.next_items.insert(previous_item, added);
            self.next_items.insert(added, first_item);
        } else {
            self.previous_items.insert(added, added);
            self.next_items.insert(added, added);
        }
        self.last = Some(added);
    }

    pub(crate) fn clear(&mut self, clear_selected: bool) {
        self.previous_items.clear();
        self.next_items.clear();
        if clear_selected {
            self.selected = None;
        }
    }

    pub(crate) fn select_previous(&mut self) -> bool {
        if let Some(selected) = self.selected {
            self.selected = self.previous_items.get(&selected).copied();
        }
        self.selected.is_some()
    }

    pub(crate) fn select_next(&mut self) -> bool {
        if let Some(selected) = self.selected {
            self.selected = self.next_items.get(&selected).copied();
        }
        self.selected.is_some()
    }
}
