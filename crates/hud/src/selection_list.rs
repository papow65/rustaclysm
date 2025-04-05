use bevy::ecs::entity::hash_map::EntityHashMap;
use bevy::prelude::{Component, Entity, KeyCode};
use keyboard::Key;
use strum::VariantArray;

#[derive(Clone, Copy, VariantArray)]
pub enum SelectionListStep {
    ManyUp,
    SingleUp,
    SingleDown,
    ManyDown,
}

impl SelectionListStep {
    const fn amount(self) -> u8 {
        if matches!(self, Self::ManyUp | Self::ManyDown) {
            10
        } else {
            1
        }
    }

    const fn is_backwards(self) -> bool {
        matches!(self, Self::ManyUp | Self::SingleUp)
    }
}

impl From<SelectionListStep> for Key {
    fn from(step: SelectionListStep) -> Self {
        Self::Code(match step {
            SelectionListStep::ManyUp => KeyCode::PageUp,
            SelectionListStep::SingleUp => KeyCode::ArrowUp,
            SelectionListStep::SingleDown => KeyCode::ArrowDown,
            SelectionListStep::ManyDown => KeyCode::PageDown,
        })
    }
}

#[derive(Default, Component)]
#[component(immutable)]
pub struct SelectionList {
    pub selected: Option<Entity>,
    first: Option<Entity>,
    last: Option<Entity>,
    previous_items: EntityHashMap<Entity>,
    next_items: EntityHashMap<Entity>,
}

impl SelectionList {
    pub fn append(&mut self, added: Entity) {
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

    pub fn clear(&mut self) {
        self.previous_items.clear();
        self.next_items.clear();
        self.first = None;
        self.last = None;
        self.selected = None;
    }

    pub fn adjust(&mut self, step: SelectionListStep) -> bool {
        for _ in 0..step.amount() {
            if let Some(selected) = self.selected {
                let sequence = if step.is_backwards() {
                    &self.previous_items
                } else {
                    &self.next_items
                };
                self.selected = sequence.get(&selected).copied();
            }
            if self.selected == self.first || self.selected == self.last {
                break;
            }
        }
        self.selected.is_some()
    }
}
