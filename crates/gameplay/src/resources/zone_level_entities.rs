use crate::ZoneLevel;
use bevy::ecs::component::ComponentHooks;
use bevy::platform::collections::{HashMap, hash_map::Entry};
use bevy::prelude::{Entity, Resource, error};

#[derive(Default, Resource)]
pub(crate) struct ZoneLevelEntities {
    zone_levels: HashMap<ZoneLevel, Entity>,
}

impl ZoneLevelEntities {
    pub(crate) fn register_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world, context| {
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
        });

        hooks.on_remove(|mut world, context| {
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
        });
    }

    pub(crate) fn get(&self, zone_level: ZoneLevel) -> Option<Entity> {
        self.zone_levels.get(&zone_level).copied()
    }
}
