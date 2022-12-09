use crate::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::cmp::Ordering;

#[derive(SystemParam)]
pub(crate) struct Envir<'w, 's> {
    pub(crate) location: ResMut<'w, Location>,
    relative_rays: Res<'w, RelativeRays>,
    floors: Query<'w, 's, &'static Floor>,
    stairs_up: Query<'w, 's, &'static StairsUp>,
    stairs_down: Query<'w, 's, &'static StairsDown>,
    obstacles: Query<'w, 's, &'static Label, With<Obstacle>>,
    opaques: Query<'w, 's, &'static Label, With<Opaque>>,
    characters: Query<'w, 's, Entity, With<Health>>,
    items: Query<'w, 's, Entity, With<Integrity>>,
}

impl<'w, 's> Envir<'w, 's> {
    // base methods

    pub(crate) fn has_floor(&self, pos: Pos) -> bool {
        self.location.any(pos, &self.floors)
    }

    pub(crate) fn stairs_up_to(&self, pos: Pos) -> Option<Pos> {
        if self.location.has_stairs_up(pos, &self.stairs_up) {
            let zone_level_up = ZoneLevel::from(pos)
                .offset(ZoneLevel {
                    x: 0,
                    level: Level::new(1),
                    z: 0,
                })
                .unwrap();

            for distance in 0..24i32 {
                for dx in -distance..=distance {
                    for dz in -distance..=distance {
                        if dx.abs().max(dz.abs()) == distance {
                            let test_up = pos
                                .offset(Pos {
                                    x: dx,
                                    level: Level::new(1),
                                    z: dz,
                                })
                                .unwrap();
                            if ZoneLevel::from(test_up) == zone_level_up
                                && self.location.has_stairs_down(test_up, &self.stairs_down)
                            {
                                return Some(test_up);
                            }
                        }
                    }
                }
            }

            panic!("No matching stairs down found");
        } else {
            None
        }
    }

    pub(crate) fn stairs_down_to(&self, pos: Pos) -> Option<Pos> {
        if self.location.has_stairs_down(pos, &self.stairs_down) {
            let zone_level_down = ZoneLevel::from(pos)
                .offset(ZoneLevel {
                    x: 0,
                    level: Level::new(-1),
                    z: 0,
                })
                .unwrap();

            // fast approach in most cases, otherwise fast enough
            for distance in 0..Zone::SIZE as i32 {
                for dx in -distance..=distance {
                    for dz in -distance..=distance {
                        if dx.abs().max(dz.abs()) == distance {
                            let test_down = pos
                                .offset(Pos {
                                    x: dx,
                                    level: Level::new(-1),
                                    z: dz,
                                })
                                .unwrap();
                            if ZoneLevel::from(test_down) == zone_level_down
                                && self.location.has_stairs_up(test_down, &self.stairs_up)
                            {
                                return Some(test_down);
                            }
                        }
                    }
                }
            }

            panic!("No matching stairs up found");
        } else {
            None
        }
    }

    pub(crate) fn find_obstacle(&self, pos: Pos) -> Option<Label> {
        self.location.get_first(pos, &self.obstacles).cloned()
    }

    fn find_opaque(&self, pos: Pos) -> Option<Label> {
        self.location.get_first(pos, &self.opaques).cloned()
    }

    pub(crate) fn find_character(&self, pos: Pos) -> Option<Entity> {
        self.location.get_first(pos, &self.characters)
    }

    pub(crate) fn find_item(&self, pos: Pos) -> Option<Entity> {
        self.location.get_first(pos, &self.items)
    }

    // helper methods

    pub(crate) fn can_see_down(&self, pos: Pos) -> bool {
        !self.has_floor(pos) || self.stairs_down_to(pos).is_some()
    }

    pub(crate) fn can_see(&self, from: Pos, to: Pos) -> Visible {
        if from == to {
            Visible::Seen
        } else if MIN_INVISIBLE_DISTANCE <= from.x.abs_diff(to.x)
            || MIN_INVISIBLE_DISTANCE <= from.z.abs_diff(to.z)
        {
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

    pub(crate) fn get_nbor(&self, from: Pos, nbor: &Nbor) -> Result<Pos, Message> {
        match nbor {
            Nbor::Up => self
                .stairs_up_to(from)
                .ok_or_else(|| Message::warn("No stairs up")),
            Nbor::Down => self
                .stairs_down_to(from)
                .ok_or_else(|| Message::warn("No stairs down")),
            horizontal => {
                let (x, z) = horizontal.horizontal_offset();
                Ok(Pos::new(from.x + x, from.level, from.z + z))
            }
        }
    }

    fn nbors(&'s self, pos: Pos) -> impl Iterator<Item = (Nbor, Pos, WalkingDistance)> + 's {
        Nbor::ALL.iter().filter_map(move |nbor| {
            self.get_nbor(pos, nbor)
                .ok()
                .map(|npos| (nbor.clone(), npos, nbor.distance()))
        })
    }

    pub(crate) fn are_nbors(&self, one: Pos, other: Pos) -> bool {
        self.nbors_if(one, move |npos| npos == other)
            .next()
            .is_some()
    }

    fn nbors_if<F>(
        &'s self,
        pos: Pos,
        acceptable: F,
    ) -> impl Iterator<Item = (Nbor, Pos, WalkingDistance)> + 's
    where
        F: 'w + 's + Fn(Pos) -> bool,
    {
        self.nbors(pos)
            .filter(move |(_nbor, npos, _distance)| acceptable(*npos))
    }

    pub(crate) fn nbors_for_moving(
        &'s self,
        pos: Pos,
        destination: Option<Pos>,
        intelligence: Intelligence,
        speed: Speed,
    ) -> impl Iterator<Item = (Pos, Milliseconds)> + 's {
        self.nbors_if(pos, move |nbor| {
            (pos.level == Level::ZERO || !self.location.all(pos).is_empty()) && {
                let at_destination = Some(nbor) == destination;
                match intelligence {
                    Intelligence::Smart => {
                        self.has_floor(nbor)
                            && (at_destination || self.find_obstacle(nbor).is_none())
                    }
                    Intelligence::Dumb => at_destination || self.find_opaque(nbor).is_none(),
                }
            }
        })
        .map(move |(_nbor, npos, distance)| (npos, distance / speed))
    }

    pub(crate) fn nbors_for_exploring(
        &'s self,
        pos: Pos,
        instruction: QueuedInstruction,
    ) -> impl Iterator<Item = Pos> + 's {
        self.nbors_if(pos, move |nbor| match instruction {
            QueuedInstruction::Attack => self.find_character(nbor).is_none(),
            QueuedInstruction::Smash => self.find_item(nbor).is_none(),
            _ => panic!("unexpected instruction {:?}", instruction),
        })
        .map(move |(_nbor, npos, _distance)| npos)
    }

    pub(crate) fn path(
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
        let estimated_duration_fn = |pos: &Pos| pos.walking_distance(to) / speed;

        //println!("dumb? {dumb:?} @{from:?}");
        match intelligence {
            Intelligence::Smart => Path::plan(from, nbors_fn, estimated_duration_fn, to),
            Intelligence::Dumb => Path::improvize(nbors_fn(&from), estimated_duration_fn, to),
        }
    }

    pub(crate) fn collide(&self, from: Pos, to: Pos, controlled: bool) -> Collision {
        assert!(from != to);
        assert!(self.are_nbors(from, to));

        match to.level.cmp(&from.level) {
            Ordering::Greater | Ordering::Less => {
                if controlled {
                    if let Some(obstacle) = self.find_obstacle(to) {
                        Collision::Blocked(obstacle)
                    } else {
                        Collision::Pass
                    }
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
