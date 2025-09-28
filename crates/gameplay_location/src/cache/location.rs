use crate::{Pos, StairsDown, StairsUp};
use bevy::ecs::query::{QueryData, QueryFilter, ROQueryItem};
use bevy::ecs::{entity::hash_map::EntityHashMap, lifecycle::HookContext, world::DeferredWorld};
use bevy::platform::collections::HashMap;
use bevy::prelude::{Entity, Query, Resource, With, error};

const NOT_FOUND: &Vec<Entity> = &Vec::new();

#[derive(Default, Resource)]
pub struct LocationCache {
    objects: HashMap<Pos, Vec<Entity>>,
    positions: EntityHashMap<Pos>,
}

impl LocationCache {
    pub(crate) fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let pos = *world
            .entity(context.entity)
            .get::<Pos>()
            .expect("Pos should be present because it was just added");
        //if let Some(faction) = world.entity(entity).get::<Faction>() {
        //    trace!("Adding {pos:?} to {faction:?} {:?}", world.entity(entity).get::<CharacterInfo>());
        //}

        let Some(mut this) = world.get_resource_mut::<Self>() else {
            error!(
                "Location missing duuring on_insert hook for {:?} @ {pos:?}",
                context.entity
            );
            return;
        };

        this.add(pos, context.entity);
        //trace!("Location: {entity:?} @ {pos:?} added");
    }

    pub(crate) fn on_replace(mut world: DeferredWorld, context: HookContext) {
        //let removed_pos = *world.entity(entity).get::<Pos>().expect("Pos should be present because it is being removed");
        //if let Some(faction) = world.entity(entity).get::<Faction>() {
        //    trace!("Removing {removed_pos:?} from {faction:?} {:?}",world.entity(entity).get::<CharacterInfo>());
        //}

        let Some(mut this) = world.get_resource_mut::<Self>() else {
            // This happens when we return from gameplay to the main menu
            //trace!("Location missing duuring on_replace hook for {entity:?}");
            return;
        };

        this.remove(context.entity);
        //trace!("Location: {entity:?} @ {pos:?} removed");
    }

    pub fn move_(&mut self, entity: Entity, to: Pos) {
        self.remove(entity);
        self.add(to, entity);
    }

    fn add(&mut self, pos: Pos, entity: Entity) {
        self.objects.entry(pos).or_default().push(entity);
        self.positions.insert(entity, pos);
    }

    fn remove(&mut self, entity: Entity) {
        if let Some(&prev_pos) = self.positions.get(&entity) {
            let old_pos_vec = self
                .objects
                .get_mut(&prev_pos)
                .expect("A vec value should be present at its previous position");
            let index = old_pos_vec
                .iter()
                .position(|&x| x == entity)
                .expect("The entity should be present at its previous position");
            old_pos_vec.swap_remove(index);
        }
        self.positions.remove(&entity);
    }

    #[must_use]
    pub fn any<'w, 's, Q, F>(&self, pos: Pos, items: &'s Query<'w, 's, Q, F>) -> bool
    where
        F: 'w + 's + QueryFilter,
        Q: 'w + 's + QueryData,
    {
        self.all(pos).any(|&x| items.get(x).is_ok())
    }

    #[must_use]
    pub fn get_first<'w, 's: 'w, Q, F>(
        &self,
        pos: Pos,
        items: &'s Query<'w, 's, Q, F>,
    ) -> Option<ROQueryItem<'w, 's, Q>>
    where
        F: 'w + 's + QueryFilter,
        Q: 'w + 's + QueryData,
    {
        self.all(pos).find_map(|&x| items.get(x).ok())
    }

    #[must_use]
    pub fn all(&self, pos: Pos) -> impl ExactSizeIterator<Item = &Entity> {
        self.objects.get(&pos).unwrap_or(NOT_FOUND).iter()
    }

    #[must_use]
    pub fn has_stairs_up<'s>(
        &self,
        from: Pos,
        stairs_up: &'s Query<'_, 's, &Pos, With<StairsUp>>,
    ) -> bool {
        from.level.up().is_some() && self.any(from, stairs_up)
    }

    #[must_use]
    pub fn has_stairs_down<'s>(
        &self,
        from: Pos,
        stairs_down: &'s Query<'_, 's, &Pos, With<StairsDown>>,
    ) -> bool {
        from.level.down().is_some() && self.any(from, stairs_down)
    }
}

#[cfg(test)]
mod location_tests {
    use crate::*;
    use bevy::prelude::*;

    const ALL_ONES: Pos = Pos::new(1, Level::new(1), 1);

    fn setup_at_origin() -> (World, Entity) {
        let mut world = World::new();
        world.init_resource::<LocationCache>();

        let location_cache = world.resource::<LocationCache>();
        assert!(location_cache.all(Pos::ORIGIN).len() == 0);

        let entity = world.spawn(Pos::ORIGIN).id();
        let location_cache = world.resource::<LocationCache>();
        assert!(location_cache.all(Pos::ORIGIN).collect::<Vec<_>>() == vec![&entity]);

        (world, entity)
    }

    #[test]
    fn test_other() {
        let (world, _) = setup_at_origin();
        let location_cache = world.resource::<LocationCache>();
        assert!(location_cache.all(ALL_ONES).len() == 0);
    }

    #[test]
    fn test_replace() {
        let (mut world, entity) = setup_at_origin();
        world
            .get_entity_mut(entity)
            .expect("Entity should be present")
            .insert(ALL_ONES);
        let location_cache = world.resource::<LocationCache>();
        assert!(location_cache.all(Pos::ORIGIN).len() == 0);
        assert!(location_cache.all(ALL_ONES).collect::<Vec<_>>() == vec![&entity]);
    }

    #[test]
    fn test_component_removal() {
        let (mut world, entity) = setup_at_origin();
        world
            .get_entity_mut(entity)
            .expect("Entity should be present")
            .remove::<Pos>();
        let location_cache = world.resource::<LocationCache>();
        assert!(location_cache.all(Pos::ORIGIN).len() == 0);
    }

    #[test]
    fn test_entity_despawn() {
        let (mut world, entity) = setup_at_origin();
        world
            .get_entity_mut(entity)
            .expect("Entity should be present")
            .despawn();
        let location_cache = world.resource::<LocationCache>();
        assert!(location_cache.all(Pos::ORIGIN).len() == 0);
    }

    #[test]
    fn test_repeat() {
        let (mut world, entity) = setup_at_origin();
        world
            .get_entity_mut(entity)
            .expect("Entity should be present")
            .insert(Pos::ORIGIN);
        let location_cache = world.resource::<LocationCache>();
        assert!(location_cache.all(Pos::ORIGIN).collect::<Vec<_>>() == vec![&entity]);
    }
}
