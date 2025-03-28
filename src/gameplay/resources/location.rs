use crate::gameplay::{Pos, StairsDown, StairsUp};
use bevy::ecs::query::{QueryData, QueryFilter, ROQueryItem};
use bevy::ecs::{component::ComponentHooks, entity::hash_map::EntityHashMap};
use bevy::platform_support::collections::HashMap;
use bevy::prelude::{Entity, Query, Resource, With, error};

const NOT_FOUND: &Vec<Entity> = &Vec::new();

#[derive(Default, Resource)]
pub(crate) struct Location {
    objects: HashMap<Pos, Vec<Entity>>,
    positions: EntityHashMap<Pos>, // TODO
}

impl Location {
    pub(crate) fn register_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(|mut world, context| {
            let pos = *world
                .entity(context.entity)
                .get::<Pos>()
                .expect("Pos should be present because it was just added");
            //if let Some(faction) = world.entity(entity).get::<Faction>() {
            //    trace!("Adding {pos:?} to {faction:?} {:?}", world.entity(entity).get::<CharacterInfo>());
            //}

            let Some(mut this) = world.get_resource_mut::<Self>() else {
                error!(
                    "Location missing duuring on_add hook for {:?} @ {pos:?}",
                    context.entity
                );
                return;
            };

            this.add(pos, context.entity);
            //trace!("Location: {entity:?} @ {pos:?} added");
        });

        hooks.on_remove(|mut world, context| {
            //let removed_pos = *world.entity(entity).get::<Pos>().expect("Pos should be present because it is being removed");
            //if let Some(faction) = world.entity(entity).get::<Faction>() {
            //    trace!("Removing {removed_pos:?} from {faction:?} {:?}",world.entity(entity).get::<CharacterInfo>());
            //}

            let Some(mut this) = world.get_resource_mut::<Self>() else {
                // This happens when we return from gameplay to the main menu
                //trace!("Location missing duuring on_remove hook for {entity:?}");
                return;
            };

            this.remove(context.entity);
            //trace!("Location: {entity:?} @ {pos:?} removed");
        });
    }

    pub(crate) fn move_(&mut self, entity: Entity, to: Pos) {
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

    pub(crate) fn any<'w, 's, Q, F>(&self, pos: Pos, items: &'s Query<'w, 's, Q, F>) -> bool
    where
        F: 'w + 's + QueryFilter,
        Q: 'w + 's + QueryData,
    {
        self.all(pos).any(|&x| items.get(x).is_ok())
    }

    pub(crate) fn get_first<'w, 's: 'w, Q, F>(
        &self,
        pos: Pos,
        items: &'s Query<'w, 's, Q, F>,
    ) -> Option<ROQueryItem<'s, Q>>
    where
        F: 'w + 's + QueryFilter,
        Q: 'w + 's + QueryData,
    {
        self.all(pos).find_map(|&x| items.get(x).ok())
    }

    pub(crate) fn all(&self, pos: Pos) -> impl ExactSizeIterator<Item = &Entity> {
        self.objects.get(&pos).unwrap_or(NOT_FOUND).iter()
    }

    pub(crate) fn has_stairs_up<'s>(
        &self,
        from: Pos,
        stairs_up: &'s Query<'_, 's, &Pos, With<StairsUp>>,
    ) -> bool {
        from.level.up().is_some() && self.any(from, stairs_up)
    }

    pub(crate) fn has_stairs_down<'s>(
        &self,
        from: Pos,
        stairs_down: &'s Query<'_, 's, &Pos, With<StairsDown>>,
    ) -> bool {
        from.level.down().is_some() && self.any(from, stairs_down)
    }
}
