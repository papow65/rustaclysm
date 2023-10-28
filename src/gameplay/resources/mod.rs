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
    pub(crate) items: Query<
        'w,
        's,
        (
            Entity,
            &'static ObjectDefinition,
            &'static ObjectName,
            Option<&'static Pos>,
            Option<&'static Amount>,
            Option<&'static Filthy>,
            &'static Containable,
            Option<&'static Parent>,
        ),
    >,
    pub(crate) containers: Query<'w, 's, (&'static Container, Option<&'static Children>)>,
    pub(crate) parents: Query<'w, 's, (Option<&'static Container>, &'static Children)>,
}
