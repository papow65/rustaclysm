use bevy::prelude::{Commands, Component, DespawnRecursiveExt, Entity, Query, States, With};
use std::marker::PhantomData;

#[derive(Default, Component)]
pub(crate) struct StateBound<T: States> {
    _phantom: PhantomData<T>,
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn despawn<T: States>(
    mut commands: Commands,
    entities: Query<Entity, With<StateBound<T>>>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
