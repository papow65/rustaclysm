use crate::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use pathfinding::prelude::{build_path, dijkstra_partial};
use std::cmp::Ordering;

#[derive(SystemParam)]
pub struct Envir<'w, 's> {
    pub location: ResMut<'w, Location>,
    relative_rays: Res<'w, RelativeRays>,
    floors: Query<'w, 's, &'static Floor>,
    stairs: Query<'w, 's, &'static Stairs>,
    obstacles: Query<'w, 's, &'static Label, With<Obstacle>>,
    opaques: Query<'w, 's, &'static Label, With<Opaque>>,
    characters: Query<'w, 's, Entity, With<Health>>,
    items: Query<'w, 's, Entity, With<Integrity>>,
}

impl<'w, 's> Envir<'w, 's> {
    // base methods

    pub fn has_floor(&self, pos: Pos) -> bool {
        self.location.any(pos, &self.floors)
    }

    pub fn has_stairs_up(&self, pos: Pos) -> bool {
        self.location.has_stairs_up(pos, &self.stairs)
    }

    pub fn has_stairs_down(&self, pos: Pos) -> bool {
        self.location.has_stairs_down(pos, &self.stairs)
    }

    pub fn find_obstacle(&self, pos: Pos) -> Option<Label> {
        self.location.get_first(pos, &self.obstacles).cloned()
    }

    fn find_opaque(&self, pos: Pos) -> Option<Label> {
        self.location.get_first(pos, &self.opaques).cloned()
    }

    pub fn find_character(&self, pos: Pos) -> Option<Entity> {
        self.location.get_first(pos, &self.characters)
    }

    pub fn find_item(&self, pos: Pos) -> Option<Entity> {
        self.location.get_first(pos, &self.items)
    }

    // helper methods

    pub fn can_see_down(&self, pos: Pos) -> bool {
        !self.has_floor(pos) || self.has_stairs_down(pos)
    }

    pub fn can_see(&self, from: Pos, to: Pos) -> Visible {
        if from == to {
            Visible::Seen
        } else if 60 < (from.x - to.x).abs() || 60 < (from.z - to.z).abs() {
            // more than 60 meter away
            Visible::Unseen
        } else {
            match self.relative_rays.ray(from, to) {
                Some((mut between, mut d_line)) => {
                    if between.all(|pos| self.find_opaque(pos).is_none())
                        && d_line.all(|(a, b)| self.can_see_down(a) || self.can_see_down(b))
                    {
                        Visible::Seen
                    } else {
                        Visible::Unseen
                    }
                }
                None => Visible::Unseen,
            }
        }
    }

    fn nbors<F>(&self, pos: Pos, acceptable: F) -> impl Iterator<Item = (Pos, Distance)> + '_
    where
        F: 'w + 's + Fn(Pos) -> bool,
    {
        pos.potential_nbors()
            .filter(move |(nbor, _)| acceptable(*nbor))
            .filter(move |(nbor, _)| {
                pos.level == nbor.level
                    || (pos.level.up() == Some(nbor.level) && self.has_stairs_up(pos))
                    || (pos.level.down() == Some(nbor.level) && self.has_stairs_down(pos))
            })
    }

    pub fn nbors_for_moving(
        &'s self,
        pos: Pos,
        destination: Option<Pos>,
        intelligence: Intelligence,
        speed: Speed,
    ) -> impl Iterator<Item = (Pos, Milliseconds)> + 's {
        self.nbors(pos, move |nbor| {
            let at_destination = Some(nbor) == destination;
            match intelligence {
                Intelligence::Smart => {
                    self.has_floor(nbor) && (at_destination || self.find_obstacle(nbor).is_none())
                }
                Intelligence::Dumb => at_destination || self.find_opaque(nbor).is_none(),
            }
        })
        .map(move |(nbor, d)| (nbor, d / speed))
    }

    pub fn nbors_for_exploring(
        &'s self,
        pos: Pos,
        instruction: QueuedInstruction,
    ) -> impl Iterator<Item = Pos> + 's {
        self.nbors(pos, move |nbor| match instruction {
            QueuedInstruction::Attack => self.find_character(nbor).is_none(),
            QueuedInstruction::Smash => self.find_item(nbor).is_none(),
            _ => panic!(),
        })
        .map(move |(nbor, _)| nbor)
    }

    /// only for smart npcs
    pub fn find_best<F>(&self, from: Pos, speed: Speed, penalty: F) -> Option<Pos>
    where
        F: Fn(&Pos) -> i64,
    {
        if penalty(&from) <= 0 {
            return None; // it's ok here
        }

        let (map, found) = dijkstra_partial(
            &from,
            |&p| {
                self.nbors_for_moving(p, None, Intelligence::Smart, speed)
                    .map(|(nbor, ms)| (nbor, ms.0 as i64 * penalty(&nbor)))
            },
            |p| penalty(p) <= 0,
        );

        found
            .or_else(|| map.keys().min_by_key(|p| penalty(p)).copied())
            .filter(|best| penalty(best) < penalty(&from))
            .map(|best| *build_path(&best, &map).get(1).unwrap())
    }

    pub fn path(
        &self,
        from: Pos,
        to: Pos,
        intelligence: Intelligence,
        speed: Speed,
    ) -> Option<Path> {
        if to == from {
            return None;
        }

        let nbors_fn = |pos: &Pos| self.nbors_for_moving(*pos, Some(to), intelligence, speed);
        let estimated_duration_fn = |pos: &Pos| pos.dist(to) / speed;

        //println!("dumb? {dumb:?} @{from:?}");
        match intelligence {
            Intelligence::Smart => Path::plan(from, nbors_fn, estimated_duration_fn, to),
            Intelligence::Dumb => Path::improvize(nbors_fn(&from), estimated_duration_fn, to),
        }
    }

    pub fn collide(&self, from: Pos, to: Pos, controlled: bool) -> Collision {
        assert!(from != to);
        assert!(to.is_potential_nbor(from));

        match to.level.cmp(&from.level) {
            x @ (Ordering::Greater | Ordering::Less) => {
                if controlled {
                    if self.has_stairs_up(if x == Ordering::Greater { from } else { to }) {
                        if let Some(obstacle) = self.find_obstacle(to) {
                            Collision::Blocked(obstacle)
                        } else {
                            Collision::Pass
                        }
                    } else if x == Ordering::Greater {
                        Collision::NoStairsUp
                    } else {
                        Collision::NoStairsDown
                    }
                //} else if x == Ordering::Greater {
                //    unimplemented!();
                } else {
                    unimplemented!();
                }
            }
            Ordering::Equal => {
                if let Some(obstacle) = self.find_obstacle(to) {
                    Collision::Blocked(obstacle)
                } else if self.has_floor(to) {
                    Collision::Pass
                } else if controlled {
                    Collision::Ledged
                } else {
                    unimplemented!();
                }
            }
        }
    }
}
