mod asset_storage;
mod camera_offset;
mod currently_visible;
mod envir;
mod expanded;
mod explored;
mod infos;
mod location;
mod map_manager;
mod map_memory_manager;
mod model_factory;
mod overmap_buffer_manager;
mod overmap_manager;
mod relative_segments;
mod spawner;
mod subzone_level_entities;
mod timeouts;
mod zone_level_entities;
mod zone_level_ids;

pub(crate) use self::{
    asset_storage::*, camera_offset::*, currently_visible::*, envir::*, expanded::*, explored::*,
    infos::*, location::*, map_manager::*, map_memory_manager::*, model_factory::*,
    overmap_buffer_manager::*, overmap_manager::*, relative_segments::*, spawner::*,
    subzone_level_entities::*, timeouts::*, zone_level_entities::*, zone_level_ids::*,
};

use crate::prelude::*;
use bevy::{ecs::system::SystemParam, prelude::*};
use std::num::Wrapping;

/// Visibility of tiles above the player character
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Resource)]
pub(crate) enum ElevationVisibility {
    #[default]
    Shown,
    Hidden,
}

/// Strategy to use when updating visualizations
#[derive(Clone, Copy, Debug, PartialEq, Eq, Resource)]
pub(crate) enum VisualizationUpdate {
    Smart,
    Forced,
}

impl VisualizationUpdate {
    pub(crate) fn forced(&self) -> bool {
        *self == Self::Forced
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::Smart;
    }
}

// pickup
#[derive(SystemParam)]
pub(crate) struct Hierarchy<'w, 's> {
    limits: Query<'w, 's, &'static ContainerLimits>,
    children: Query<'w, 's, &'static Children>,
    items: Query<'w, 's, Item>,
}

impl<'w, 's> Hierarchy<'w, 's> {
    pub(crate) fn items_in(&self, container: Entity) -> impl Iterator<Item = ItemItem> + '_ {
        self.children
            .iter_descendants(container)
            .flat_map(|item| self.items.get(item))
    }

    pub(crate) fn container(&self, container_entity: Entity) -> &ContainerLimits {
        self.limits
            .get(container_entity)
            .expect("An existing container")
    }
}

#[derive(Debug, Default, Resource)]
pub(crate) struct GameplayCounter(pub(crate) Wrapping<usize>);

#[derive(SystemParam)]
pub(crate) struct GameplaySession<'w, 's> {
    current: Res<'w, GameplayCounter>,
    last: Local<'s, Wrapping<usize>>,
}

impl<'w, 's> GameplaySession<'w, 's> {
    pub(crate) fn is_changed(&mut self) -> bool {
        let reset = self.current.0 != *self.last;
        *self.last = self.current.0;
        reset
    }
}
