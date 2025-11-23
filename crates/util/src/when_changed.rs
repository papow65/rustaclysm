use bevy::prelude::{Changed, Component, Query};

#[must_use]
pub fn when_changed<C: Component>(changed: Query<(), Changed<C>>) -> bool {
    !changed.is_empty()
}
