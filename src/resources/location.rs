use bevy::ecs::query::{Fetch, FilterFetch, ReadOnlyFetch, WorldQuery};
use bevy::prelude::{Entity, Query};
use bevy::utils::HashMap;

use super::super::components::*;

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
                //println!("\n\rTogether {:?}", vec);
            } else {
                self.all.insert(pos, vec![entity]);
            }
            self.reverse.insert(entity, pos);
        } else {
            self.reverse.remove(&entity);
        }
    }

    pub fn any<'a, Q, F>(&self, pos: Pos, items: &'a Query<'a, Q, F>) -> bool
    where
        F: 'a,
        Q: 'a,
        Q: WorldQuery,
        F: WorldQuery,
        <Q as WorldQuery>::Fetch: ReadOnlyFetch,
        <F as WorldQuery>::Fetch: FilterFetch,
    {
        self.all
            .get(&pos)
            .unwrap_or(&vec![])
            .iter()
            .any(|&x| items.get(x).is_ok())
    }

    pub fn get_first<'a, Q, F>(
        &self,
        pos: Pos,
        items: &'a Query<'a, Q, F>,
    ) -> Option<<<Q as WorldQuery>::Fetch as Fetch<'a>>::Item>
    where
        F: 'a,
        Q: 'a,
        F: WorldQuery,
        Q: WorldQuery,
        <Q as WorldQuery>::Fetch: ReadOnlyFetch,
        <F as WorldQuery>::Fetch: FilterFetch,
    {
        self.all
            .get(&pos)
            .unwrap_or(&vec![])
            .iter()
            .find_map(|&x| items.get(x).ok())
    }

    // helper methods

    pub fn has_stairs_up<'a>(&self, from: Pos, stairs: &'a Query<'a, &'static Stairs>) -> bool {
        self.any(from, stairs)
    }

    pub fn has_stairs_down<'a>(&self, from: Pos, stairs: &'a Query<'a, &'static Stairs>) -> bool {
        let below = Pos(from.0, from.1 - 1, from.2);
        self.any(below, stairs)
    }
}
