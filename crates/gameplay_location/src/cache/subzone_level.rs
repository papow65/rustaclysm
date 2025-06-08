use crate::SubzoneLevel;
use bevy::ecs::component::ComponentHooks;
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

    pub(crate) fn register_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world, context| {
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
        });

        hooks.on_remove(|mut world, context| {
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
        });
    }
}
