use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::{Component, Entity, KeyCode};

#[derive(Clone, Copy)]
pub(crate) enum StepSize {
    Single,
    Many,
}

impl StepSize {
    const fn amount(&self) -> u8 {
        if matches!(self, Self::Many) {
            10
        } else {
            1
        }
    }
}

impl From<&KeyCode> for StepSize {
    fn from(key: &KeyCode) -> Self {
        match key {
            KeyCode::ArrowUp | KeyCode::ArrowDown => Self::Single,
            KeyCode::PageUp | KeyCode::PageDown => Self::Many,
            _ => panic!("Unsupported conversion to StepSize for {key:?}"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum StepDirection {
    Up,
    Down,
}

impl From<&KeyCode> for StepDirection {
    fn from(key: &KeyCode) -> Self {
        match key {
            KeyCode::ArrowUp | KeyCode::PageUp => Self::Up,
            KeyCode::ArrowDown | KeyCode::PageDown => Self::Down,
            _ => panic!("Unsupported conversion to StepDirection for {key:?}"),
        }
    }
}

#[derive(Default, Component)]
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

    pub(crate) fn clear(&mut self) {
        self.previous_items.clear();
        self.next_items.clear();
        self.first = None;
        self.last = None;
        self.selected = None;
    }

    pub(crate) fn adjust(&mut self, size: StepSize, direction: StepDirection) -> bool {
        for _ in 0..size.amount() {
            if let Some(selected) = self.selected {
                let sequence = if direction == StepDirection::Up {
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
