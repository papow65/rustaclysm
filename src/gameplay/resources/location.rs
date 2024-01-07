use crate::prelude::{Pos, StairsDown, StairsUp};
use bevy::{
    ecs::query::{ROQueryItem, ReadOnlyWorldQuery},
    prelude::{Entity, Query, Resource, With},
    utils::HashMap,
};

const NOT_FOUND: &Vec<Entity> = &Vec::new();

#[derive(Default, Resource)]
pub(crate) struct Location {
    objects: HashMap<Pos, Vec<Entity>>,
    positions: HashMap<Entity, Pos>,
}

impl Location {
    pub(crate) fn update(&mut self, entity: Entity, pos: Option<Pos>) {
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

        if let Some(pos) = pos {
            if let Some(vec) = self.objects.get_mut(&pos) {
                assert!(
                    !vec.contains(&entity),
                    "The given entity shouldn't alrady be at this position"
                );
                vec.push(entity);
                //println!("\n\rTogether {vec:?}");
            } else {
                self.objects.insert(pos, vec![entity]);
            }
            self.positions.insert(entity, pos);
        } else {
            self.positions.remove(&entity);
        }
    }

    fn entities(&self, pos: Pos) -> impl ExactSizeIterator<Item = &Entity> {
        self.objects.get(&pos).unwrap_or(NOT_FOUND).iter()
    }

    pub(crate) fn any<'w, 's, Q, F>(&self, pos: Pos, items: &'s Query<'w, 's, Q, F>) -> bool
    where
        F: 'w + 's + ReadOnlyWorldQuery,
        Q: 'w + 's + ReadOnlyWorldQuery,
    {
        self.entities(pos).any(|&x| items.get(x).is_ok())
    }

    pub(crate) fn get_first<'w, 's: 'w, Q, F>(
        &self,
        pos: Pos,
        items: &'s Query<'w, 's, Q, F>,
    ) -> Option<ROQueryItem<'s, Q>>
    where
        F: 'w + 's + ReadOnlyWorldQuery,
        Q: 'w + 's + ReadOnlyWorldQuery,
    {
        self.entities(pos).find_map(|&x| items.get(x).ok())
    }

    pub(crate) fn all(&self, pos: Pos) -> Vec<Entity> {
        self.entities(pos).copied().collect()
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
