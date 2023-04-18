mod envir;
mod explored;
mod infos;
mod instruction_queue;
mod location;
mod model_factory;
mod player_action_state;
mod relative_segments;
mod spawner;
mod subzone_level_entities;
mod timeouts;
mod zone_level_entities;
mod zone_level_ids;

pub(crate) use self::{
    envir::*, explored::*, infos::*, instruction_queue::*, location::*, model_factory::*,
    player_action_state::*, relative_segments::*, spawner::*, subzone_level_entities::*,
    timeouts::*, zone_level_entities::*, zone_level_ids::*,
};

use crate::prelude::*;
use bevy::{ecs::system::SystemParam, prelude::*};

//** Visibility of tiles above the player character */
#[derive(Clone, Copy, Debug, PartialEq, Eq, Resource)]
pub(crate) enum ElevationVisibility {
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
    pub(crate) state: TextSection,
    pub(crate) details: Vec<TextSection>,
}

#[derive(Debug, Resource)]
pub(crate) struct CameraOffset(Vec3);

impl CameraOffset {
    pub(crate) fn zoom(&mut self, zoom_direction: ZoomDirection) {
        self.0 *= 0.75_f32.powi(if zoom_direction == ZoomDirection::In {
            1
        } else {
            -1
        });
    }

    pub(crate) fn offset(&self) -> Vec3 {
        self.0
    }
}

impl Default for CameraOffset {
    fn default() -> Self {
        Self(Vec3::new(0.0, 28.4, 35.5))
    }
}

// pickup
#[derive(SystemParam)]
pub(crate) struct Hierarchy<'w, 's> {
    pub(crate) items: Query<
        'w,
        's,
        (
            Entity,
            &'static ObjectName,
            Option<&'static Pos>,
            Option<&'static Amount>,
            Option<&'static Filthy>,
            &'static Containable,
            Option<&'static Parent>,
        ),
    >,
    pub(crate) containers: Query<'w, 's, &'static Container>,
}

#[derive(SystemParam)]
pub(crate) struct Actors<'w, 's> {
    pub(crate) q: Query<'w, 's, ActorTuple<'static>>,
}

impl<'w, 's> Actors<'w, 's> {
    pub(crate) fn actors(&'s self) -> impl Iterator<Item = Actor<'s>> {
        self.q.iter().map(Actor::from)
    }

    pub(crate) fn get(&'s self, entity: Entity) -> Actor<'s> {
        self.q.get(entity).map(Actor::from).unwrap()
    }

    pub(crate) fn collect_factions(&'s self) -> Vec<(Pos, &'s Faction)> {
        self.q
            .iter()
            .map(|(_, _, p, _, _, f, ..)| (*p, f))
            .collect::<Vec<(Pos, &'s Faction)>>()
    }
}
