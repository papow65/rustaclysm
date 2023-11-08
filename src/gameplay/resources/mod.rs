mod camera_offset;
mod envir;
mod explored;
mod infos;
mod instruction_queue;
mod location;
mod map_manager;
mod model_factory;
mod overmap_buffer_manager;
mod overmap_manager;
mod player_action_state;
mod relative_segments;
mod spawner;
mod subzone_level_entities;
mod timeouts;
mod zone_level_entities;
mod zone_level_ids;

pub(crate) use self::{
    camera_offset::*, envir::*, explored::*, infos::*, instruction_queue::*, location::*,
    map_manager::*, model_factory::*, overmap_buffer_manager::*, overmap_manager::*,
    player_action_state::*, relative_segments::*, spawner::*, subzone_level_entities::*,
    timeouts::*, zone_level_entities::*, zone_level_ids::*,
};

use crate::prelude::*;
use bevy::{ecs::system::SystemParam, prelude::*};
use std::num::Wrapping;

//** Visibility of tiles above the player character */
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Resource)]
pub(crate) enum ElevationVisibility {
    #[default]
    Shown,
    Hidden,
}

//** Strategy to use when updating visualizations */
#[derive(Clone, Copy, Debug, PartialEq, Eq, Resource)]
pub(crate) enum VisualizationUpdate {
    Smart,
    Forced,
}

#[derive(Debug, Default, Resource)]
pub(crate) struct StatusTextSections {
    pub(crate) fps: TextSection,
    pub(crate) time: TextSection,
    pub(crate) health: [TextSection; 2],
    pub(crate) stamina: [TextSection; 2],
    pub(crate) speed: [TextSection; 3],
    pub(crate) player_action_state: TextSection,
    pub(crate) wielded: Vec<TextSection>,
    pub(crate) enemies: Vec<TextSection>,
    pub(crate) details: Vec<TextSection>,
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
