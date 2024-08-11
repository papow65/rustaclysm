use bevy::prelude::{on_event, resource_exists, Condition, Event, Events};

pub(crate) fn on_safe_event<T: Event>() -> impl Condition<()> {
    resource_exists::<Events<T>>.and_then(on_event::<T>())
}
