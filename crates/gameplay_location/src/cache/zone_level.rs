use crate::ZoneLevel;
use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
use bevy::platform::collections::{HashMap, hash_map::Entry};
use bevy::prelude::{Entity, Resource, error};

#[derive(Default, Resource)]
pub struct ZoneLevelCache {
    zone_levels: HashMap<ZoneLevel, Entity>,
}

impl ZoneLevelCache {
    pub(crate) fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let zone_level = *world
            .entity(context.entity)
            .get::<ZoneLevel>()
            .expect("ZoneLevel should be present because it was just added");
        if let Some(mut this) = world.get_resource_mut::<Self>() {
            let entry = this.zone_levels.entry(zone_level);
            assert!(
                matches!(entry, Entry::Vacant(..)),
                "Duplicate for {entry:?} - new: {:?}",
                context.entity
            );
            entry.insert(context.entity);
        }
    }

    pub(crate) fn on_replace(mut world: DeferredWorld, context: HookContext) {
        let removed_zone_level = *world
            .entity(context.entity)
            .get::<ZoneLevel>()
            .expect("ZoneLevel should be present because it is being removed");
        if let Some(mut this) = world.get_resource_mut::<Self>() {
            let entry = this.zone_levels.entry(removed_zone_level);
            match entry {
                Entry::Occupied(occupied) => {
                    occupied.remove();
                }
                Entry::Vacant(..) => {
                    error!("The removed zone level entity should have been found");
                }
            }
        }
    }

    #[must_use]
    pub fn get(&self, zone_level: ZoneLevel) -> Option<Entity> {
        self.zone_levels.get(&zone_level).copied()
    }
}

#[cfg(test)]
mod zone_level_tests {
    use crate::*;
    use bevy::prelude::*;

    const ORIGIN: ZoneLevel = ZoneLevel {
        zone: Zone { x: 0, z: 0 },
        level: Level::ZERO,
    };
    const ALL_ONES: ZoneLevel = ZoneLevel {
        zone: Zone { x: 1, z: 1 },
        level: Level::new(1),
    };

    fn setup_at_origin() -> (World, Entity) {
        let mut world = World::new();
        world.init_resource::<ZoneLevelCache>();

        let zone_level_cache = world.resource::<ZoneLevelCache>();
        assert!(zone_level_cache.get(ORIGIN).is_none());

        let entity = world.spawn(ORIGIN).id();
        let zone_level_cache = world.resource::<ZoneLevelCache>();
        assert!(zone_level_cache.get(ORIGIN) == Some(entity));

        (world, entity)
    }

    #[test]
    fn test_plain() {
        let (world, _) = setup_at_origin();
        let zone_level_cache = world.resource::<ZoneLevelCache>();
        assert!(zone_level_cache.get(ALL_ONES).is_none());
    }

    #[test]
    fn test_replace() {
        let (mut world, entity) = setup_at_origin();
        world
            .get_entity_mut(entity)
            .expect("Entity should be present")
            .insert(ALL_ONES);
        let zone_level_cache = world.resource::<ZoneLevelCache>();
        assert!(zone_level_cache.get(ORIGIN).is_none());
        assert!(zone_level_cache.get(ALL_ONES) == Some(entity));
    }

    #[test]
    fn test_component_removal() {
        let (mut world, entity) = setup_at_origin();
        world
            .get_entity_mut(entity)
            .expect("Entity should be present")
            .remove::<ZoneLevel>();
        let zone_level_cache = world.resource::<ZoneLevelCache>();
        assert!(zone_level_cache.get(ORIGIN).is_none());
    }

    #[test]
    fn test_entity_despawn() {
        let (mut world, entity) = setup_at_origin();
        world
            .get_entity_mut(entity)
            .expect("Entity should be present")
            .despawn();
        let zone_level_cache = world.resource::<ZoneLevelCache>();
        assert!(zone_level_cache.get(ORIGIN).is_none());
    }

    #[test]
    fn test_repeat() {
        let (mut world, entity) = setup_at_origin();
        world
            .get_entity_mut(entity)
            .expect("Entity should be present")
            .insert(ORIGIN);
        let zone_level_cache = world.resource::<ZoneLevelCache>();
        assert!(zone_level_cache.get(ORIGIN) == Some(entity));
    }
}
