use crate::SubzoneLevel;
use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
use bevy::platform::collections::{HashMap, hash_map::Entry};
use bevy::prelude::{Entity, Resource, error};

#[derive(Default, Resource)]
pub struct SubzoneLevelCache {
    subzone_levels: HashMap<SubzoneLevel, Entity>,
}

impl SubzoneLevelCache {
    #[must_use]
    pub fn get(&self, subzone_level: SubzoneLevel) -> Option<Entity> {
        self.subzone_levels.get(&subzone_level).copied()
    }

    #[must_use]
    pub fn loaded(&self) -> bool {
        !self.subzone_levels.is_empty()
    }

    pub(crate) fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let subzone_level = *world
            .entity(context.entity)
            .get::<SubzoneLevel>()
            .expect("SubzoneLevel should be present because it was just added");
        if let Some(mut this) = world.get_resource_mut::<Self>() {
            let entry = this.subzone_levels.entry(subzone_level);
            assert!(
                matches!(entry, Entry::Vacant(..)),
                "There shouldn't be an existing entry"
            );
            entry.insert(context.entity);
        }
    }

    pub(crate) fn on_replace(mut world: DeferredWorld, context: HookContext) {
        let removed_subzone_level = *world
            .entity(context.entity)
            .get::<SubzoneLevel>()
            .expect("SubzoneLevel should be present because it is being removed");
        if let Some(mut this) = world.get_resource_mut::<Self>() {
            let entry = this.subzone_levels.entry(removed_subzone_level);
            match entry {
                Entry::Occupied(occupied) => {
                    occupied.remove();
                }
                Entry::Vacant(..) => {
                    error!("The removed subzone level entity should have been found");
                }
            }
        }
    }
}

#[cfg(test)]
mod subzone_level_tests {
    use crate::*;
    use bevy::prelude::*;

    const ORIGIN: SubzoneLevel = SubzoneLevel {
        x: 0,
        level: Level::ZERO,
        z: 0,
    };
    const ALL_ONES: SubzoneLevel = SubzoneLevel {
        x: 1,
        level: Level::new(1),
        z: 1,
    };

    fn setup_at_origin() -> (World, Entity) {
        let mut world = World::new();
        world.init_resource::<SubzoneLevelCache>();

        let subzone_level_cache = world.resource::<SubzoneLevelCache>();
        assert!(subzone_level_cache.get(ORIGIN).is_none());

        let entity = world.spawn(ORIGIN).id();
        let subzone_level_cache = world.resource::<SubzoneLevelCache>();
        assert!(subzone_level_cache.get(ORIGIN) == Some(entity));

        (world, entity)
    }

    #[test]
    fn test_plain() {
        let (world, _) = setup_at_origin();
        let subzone_level_cache = world.resource::<SubzoneLevelCache>();
        assert!(subzone_level_cache.get(ALL_ONES).is_none());
    }

    #[test]
    fn test_replace() {
        let (mut world, entity) = setup_at_origin();
        world
            .get_entity_mut(entity)
            .expect("Entity should be present")
            .insert(ALL_ONES);
        let subzone_level_cache = world.resource::<SubzoneLevelCache>();
        assert!(subzone_level_cache.get(ORIGIN).is_none());
        assert!(subzone_level_cache.get(ALL_ONES) == Some(entity));
    }

    #[test]
    fn test_component_removal() {
        let (mut world, entity) = setup_at_origin();
        world
            .get_entity_mut(entity)
            .expect("Entity should be present")
            .remove::<SubzoneLevel>();
        let subzone_level_cache = world.resource::<SubzoneLevelCache>();
        assert!(subzone_level_cache.get(ORIGIN).is_none());
    }

    #[test]
    fn test_entity_despawn() {
        let (mut world, entity) = setup_at_origin();
        world
            .get_entity_mut(entity)
            .expect("Entity should be present")
            .despawn();
        let subzone_level_cache = world.resource::<SubzoneLevelCache>();
        assert!(subzone_level_cache.get(ORIGIN).is_none());
    }

    #[test]
    fn test_repeat() {
        let (mut world, entity) = setup_at_origin();
        world
            .get_entity_mut(entity)
            .expect("Entity should be present")
            .insert(ORIGIN);
        let subzone_level_cache = world.resource::<SubzoneLevelCache>();
        assert!(subzone_level_cache.get(ORIGIN) == Some(entity));
    }
}
