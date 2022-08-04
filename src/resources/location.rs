use bevy::ecs::query::{ROQueryItem, WorldQuery};
use bevy::prelude::{Entity, Query};
use bevy::utils::HashMap;

use crate::components::{Pos, Stairs};

const NOT_FOUND: &Vec<Entity> = &Vec::new();

pub struct Location {
    all: HashMap<Pos, Vec<Entity>>,
    reverse: HashMap<Entity, Pos>,
}

impl Location {
    // base methods

    pub fn new() -> Self {
        Self {
            all: HashMap::default(),
            reverse: HashMap::default(),
        }
    }

    pub fn update(&mut self, entity: Entity, pos: Option<Pos>) {
        if let Some(&prev_pos) = self.reverse.get(&entity) {
            let old_pos_vec = self.all.get_mut(&prev_pos).unwrap();
            let index = old_pos_vec.iter().position(|&x| x == entity).unwrap();
            old_pos_vec.swap_remove(index);
        }

        if let Some(pos) = pos {
            if let Some(vec) = self.all.get_mut(&pos) {
                assert!(!vec.iter().any(|&x| x == entity));
                vec.push(entity);
                //println!("\n\rTogether {vec:?}");
            } else {
                self.all.insert(pos, vec![entity]);
            }
            self.reverse.insert(entity, pos);
        } else {
            self.reverse.remove(&entity);
        }
    }

    fn entities<'l>(&'l self, pos: Pos) -> impl ExactSizeIterator<Item = &Entity> {
        self.all.get(&pos).unwrap_or(NOT_FOUND).iter()
    }

    pub fn any<'w, 's, Q, F>(&self, pos: Pos, items: &'s Query<'w, 's, Q, F>) -> bool
    where
        F: 'w + 's + WorldQuery,
        Q: 'w + 's + WorldQuery,
    {
        self.entities(pos).any(|&x| items.get(x).is_ok())
    }

    pub fn get_first<'w, 's: 'w, Q, F>(
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

    pub fn exists(&self, pos: Pos) -> bool {
        0 < self.entities(pos).len()
    }

    pub fn all(&self, pos: Pos) -> Vec<Entity> {
        self.entities(pos).cloned().collect()
    }

    // helper methods

    pub fn has_stairs_up<'w, 's>(
        &self,
        from: Pos,
        stairs: &'s Query<'w, 's, &'static Stairs>,
    ) -> bool {
        self.any(from, stairs)
    }

    pub fn has_stairs_down<'w, 's>(
        &self,
        from: Pos,
        stairs: &'s Query<'w, 's, &'static Stairs>,
    ) -> bool {
        let below = Pos(from.0, from.1 - 1, from.2);
        self.any(below, stairs)
    }
}
