use crate::prelude::ZoneLevel;
use bevy::{
    ecs::component::ComponentHooks,
    prelude::{Entity, Resource},
    utils::{hashbrown::hash_map::Entry, HashMap},
};

#[derive(Default, Resource)]
pub(crate) struct ZoneLevelEntities {
    zone_levels: HashMap<ZoneLevel, Entity>,
}

impl ZoneLevelEntities {
    pub(crate) fn register_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world, entity, _component_id| {
            let zone_level = *world
                .entity(entity)
                .get::<ZoneLevel>()
                .expect("ZoneLevel should be present because it was just added");
            if let Some(mut this) = world.get_resource_mut::<Self>() {
                let entry = this.zone_levels.entry(zone_level);
                assert!(
                    matches!(entry, Entry::Vacant(..)),
                    "Duplicate for {entry:?} - new: {entity:?}"
                );
                entry.insert(entity);
            }
        });

        hooks.on_remove(|mut world, entity, _component_id| {
            let removed_zone_level = *world
                .entity(entity)
                .get::<ZoneLevel>()
                .expect("ZoneLevel should be present because it is being removed");
            if let Some(mut this) = world.get_resource_mut::<Self>() {
                let entry = this.zone_levels.entry(removed_zone_level);
                match entry {
                    Entry::Occupied(occupied) => {
                        occupied.remove();
                    }
                    Entry::Vacant(..) => {
                        eprintln!("The removed zone level entity should have been found");
                    }
                }
            }
        });
    }

    pub(crate) fn get(&self, zone_level: ZoneLevel) -> Option<Entity> {
        self.zone_levels.get(&zone_level).copied()
    }
}
