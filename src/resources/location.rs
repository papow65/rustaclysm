use crate::prelude::{Pos, StairsDown, StairsUp};
use bevy::ecs::query::{ROQueryItem, WorldQuery};
use bevy::prelude::{Entity, Query};
use bevy::utils::HashMap;

const NOT_FOUND: &Vec<Entity> = &Vec::new();

#[derive(Default)]
pub(crate) struct Location {
    objects: HashMap<Pos, Vec<Entity>>,
    positions: HashMap<Entity, Pos>,
}

impl Location {
    pub(crate) fn update(&mut self, entity: Entity, pos: Option<Pos>) {
        if let Some(&prev_pos) = self.positions.get(&entity) {
            let old_pos_vec = self.objects.get_mut(&prev_pos).unwrap();
            let index = old_pos_vec.iter().position(|&x| x == entity).unwrap();
            old_pos_vec.swap_remove(index);
        }

        if let Some(pos) = pos {
            if let Some(vec) = self.objects.get_mut(&pos) {
                assert!(!vec.iter().any(|&x| x == entity));
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

    fn entities<'l>(&'l self, pos: Pos) -> impl ExactSizeIterator<Item = &Entity> {
        self.objects.get(&pos).unwrap_or(NOT_FOUND).iter()
    }

    pub(crate) fn any<'w, 's, Q, F>(&self, pos: Pos, items: &'s Query<'w, 's, Q, F>) -> bool
    where
        F: 'w + 's + WorldQuery,
        Q: 'w + 's + WorldQuery,
    {
        self.entities(pos).any(|&x| items.get(x).is_ok())
    }

    pub(crate) fn get_first<'w, 's: 'w, Q, F>(
        &self,
        pos: Pos,
        items: &'s Query<'w, 's, Q, F>,
    ) -> Option<ROQueryItem<'s, Q>>
    where
        F: 'w + 's + WorldQuery,
        Q: 'w + 's + WorldQuery,
    {
        self.entities(pos).find_map(|&x| items.get(x).ok())
    }

    pub(crate) fn exists(&self, pos: Pos) -> bool {
        0 < self.entities(pos).len()
    }

    pub(crate) fn all(&self, pos: Pos) -> Vec<Entity> {
        self.entities(pos).copied().collect()
    }

    pub(crate) fn has_stairs_up<'w, 's>(
        &self,
        from: Pos,
        stairs_up: &'s Query<'w, 's, &'static StairsUp>,
    ) -> bool {
        from.level.up().is_some() && self.any(from, stairs_up)
    }

    pub(crate) fn has_stairs_down<'w, 's>(
        &self,
        from: Pos,
        stairs_down: &'s Query<'w, 's, &'static StairsDown>,
    ) -> bool {
        from.level.down().is_some() && self.any(from, stairs_down)
    }
}
