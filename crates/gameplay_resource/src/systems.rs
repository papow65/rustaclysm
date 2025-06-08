use bevy::prelude::{Commands, Resource};

pub(crate) fn add_resource<T: Default + Resource>(mut commands: Commands) {
    commands.insert_resource(T::default());
}

pub(crate) fn remove_resource<T: Resource>(mut commands: Commands) {
    commands.remove_resource::<T>();
}
