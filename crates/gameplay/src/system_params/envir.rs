use crate::{
    Accessible, Amount, Closeable, Corpse, Health, HorizontalDirection, Hurdle, Intelligence, Item,
    ItemItem, Level, LevelOffset, Life, Location, Nbor, NborDistance, ObjectName, Obstacle, Opaque,
    OpaqueFloor, Openable, Pos, PosOffset, QueuedInstruction, StairsDown, StairsUp,
    StandardIntegrity, WalkingCost, Zone, ZoneLevel,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Entity, Query, ResMut, With, Without, debug, warn};
use cdda_json_files::MoveCost;
use pathfinding::prelude::astar;
use std::cmp::Ordering;
use units::{Duration, Speed};

pub(crate) enum Collision<'a> {
    Pass,
    //Fall(Pos), // todo
    Blocked(&'a ObjectName),
    Ledged,
    Opened(Entity),
}

#[derive(SystemParam)]
pub(crate) struct Envir<'w, 's> {
    pub(crate) location: ResMut<'w, Location>,
    accessibles: Query<'w, 's, &'static Accessible>,
    hurdles: Query<'w, 's, &'static Hurdle>,
    openables: Query<'w, 's, (Entity, &'static ObjectName), With<Openable>>,
    closeables: Query<'w, 's, (Entity, &'static ObjectName), With<Closeable>>,
    stairs_up: Query<'w, 's, &'static Pos, With<StairsUp>>,
    stairs_down: Query<'w, 's, &'static Pos, With<StairsDown>>,
    terrain: Query<'w, 's, &'static ObjectName, (Without<Health>, Without<Amount>)>,
    obstacles: Query<'w, 's, &'static ObjectName, With<Obstacle>>,
    opaques: Query<'w, 's, &'static ObjectName, With<Opaque>>,
    opaque_floors: Query<'w, 's, &'static OpaqueFloor>,
    characters: Query<'w, 's, (Entity, &'static ObjectName), With<Life>>,
    smashables: Query<'w, 's, Entity, (With<StandardIntegrity>, Without<Corpse>)>,
    pulpables: Query<'w, 's, Entity, (With<StandardIntegrity>, With<Corpse>)>,
    items: Query<'w, 's, Item>,
}

impl<'w, 's> Envir<'w, 's> {
    // base methods

    pub(crate) fn is_accessible(&self, pos: Pos) -> bool {
        self.location.any(pos, &self.accessibles)
    }

    pub(crate) fn is_water(&self, pos: Pos) -> bool {
        self.location
            .get_first(pos, &self.accessibles)
            .map(|floor| floor.water)
            == Some(true)
    }

    pub(crate) fn stairs_up_to(&self, from: Pos) -> Option<Pos> {
        if self.location.has_stairs_up(from, &self.stairs_up) {
            let zone_level_up = ZoneLevel::from(from).offset(LevelOffset::UP)?;

            for distance in 0..24_i32 {
                for dx in -distance..=distance {
                    for dz in -distance..=distance {
                        if dx.abs().max(dz.abs()) == distance {
                            let test_up = from
                                .offset(PosOffset {
                                    x: dx,
                                    level: LevelOffset::UP,
                                    z: dz,
                                })
                                .expect("Valid position above");
                            if ZoneLevel::from(test_up) == zone_level_up
                                && self.location.has_stairs_down(test_up, &self.stairs_down)
                            {
                                return Some(test_up);
                            }
                        }
                    }
                }
            }

            warn!("No matching stairs up found");
            from.offset(PosOffset {
                x: 0,
                level: LevelOffset::UP,
                z: 0,
            })
        } else {
            None
        }
    }

    pub(crate) fn stairs_down_to(&self, from: Pos) -> Option<Pos> {
        if self.location.has_stairs_down(from, &self.stairs_down) {
            let zone_level_down = ZoneLevel::from(from).offset(LevelOffset::DOWN)?;

            // fast approach in most cases, otherwise fast enough
            for distance in 0..Zone::SIZE {
                for dx in -distance..=distance {
                    for dz in -distance..=distance {
                        if dx.abs().max(dz.abs()) == distance {
                            let test_down = from
                                .offset(PosOffset {
                                    x: dx,
                                    level: LevelOffset::DOWN,
                                    z: dz,
                                })
                                .expect("Valid position below");
                            if ZoneLevel::from(test_down) == zone_level_down
                                && self.location.has_stairs_up(test_down, &self.stairs_up)
                            {
                                return Some(test_down);
                            }
                        }
                    }
                }
            }

            warn!("No matching stairs down found");
            from.offset(PosOffset {
                x: 0,
                level: LevelOffset::DOWN,
                z: 0,
            })
        } else {
            None
        }
    }

    pub(crate) fn find_openable(&self, pos: Pos) -> Option<(Entity, &ObjectName)> {
        self.location.get_first(pos, &self.openables)
    }

    pub(crate) fn find_closeable(&self, pos: Pos) -> Option<(Entity, &ObjectName)> {
        self.location.get_first(pos, &self.closeables)
    }

    pub(crate) fn find_terrain(&self, pos: Pos) -> Option<&ObjectName> {
        self.location.get_first(pos, &self.terrain)
    }

    pub(crate) fn find_obstacle(&self, pos: Pos) -> Option<&ObjectName> {
        self.location.get_first(pos, &self.obstacles)
    }

    pub(crate) fn is_opaque(&self, pos: Pos) -> bool {
        self.location.any(pos, &self.opaques)
    }

    pub(crate) fn has_opaque_floor(&self, pos: Pos) -> bool {
        self.location.any(pos, &self.opaque_floors)
    }

    pub(crate) fn find_character(&self, pos: Pos) -> Option<(Entity, &ObjectName)> {
        self.location.get_first(pos, &self.characters)
    }

    pub(crate) fn find_smashable(&self, pos: Pos) -> Option<Entity> {
        self.location.get_first(pos, &self.smashables)
    }

    pub(crate) fn find_pulpable(&self, pos: Pos) -> Option<Entity> {
        self.location.get_first(pos, &self.pulpables)
    }

    pub(crate) fn find_item(&self, pos: Pos) -> Option<ItemItem> {
        self.location.get_first(pos, &self.items)
    }

    pub(crate) fn all_items(&self, pos: Pos) -> impl Iterator<Item = ItemItem> + use<'_> {
        self.location
            .all(pos)
            .flat_map(|&entity| self.items.get(entity))
    }

    // helper methods

    /// In case of vertical nbors: Follow stairs, even when they do not go staight up or down. Without stairs, see the raw position below/above, unless that contains a stair to somewhere else.
    #[expect(dead_code)]
    pub(crate) fn get_looking_nbor(&self, from: Pos, nbor: Nbor) -> Option<Pos> {
        match nbor {
            Nbor::Up => self.stairs_up_to(from).or_else(|| from.raw_nbor(Nbor::Up)),
            Nbor::Down => self
                .stairs_down_to(from)
                .or_else(|| from.raw_nbor(Nbor::Down)),
            Nbor::Horizontal(horizontal_direction) => {
                // No stairs
                Some(from.horizontal_nbor(horizontal_direction))
            }
        }
    }

    /// Follow stairs, even when they do not go staight up or down.
    pub(crate) fn get_nbor(&self, from: Pos, nbor: Nbor) -> Result<Pos, &str> {
        match nbor {
            Nbor::Up => self.stairs_up_to(from).ok_or("No stairs up"),
            Nbor::Down => self.stairs_down_to(from).ok_or("No stairs down"),
            Nbor::Horizontal(horizontal_direction) => {
                // No stairs
                Ok(from.horizontal_nbor(horizontal_direction))
            }
        }
    }

    /// Follow stairs, even when they do not go staight up or down.
    pub(crate) fn to_nbor(&self, from: Pos, to: Pos) -> Option<Nbor> {
        let offset = to - from;
        match offset.level {
            LevelOffset::UP if self.get_nbor(from, Nbor::Up) == Ok(to) => Some(Nbor::Up),
            LevelOffset::ZERO => HorizontalDirection::try_from(offset)
                .ok()
                .map(Nbor::Horizontal),
            LevelOffset::DOWN if self.get_nbor(from, Nbor::Down) == Ok(to) => Some(Nbor::Down),
            _ => None,
        }
    }

    fn nbors(&'s self, pos: Pos) -> impl Iterator<Item = (Nbor, Pos, WalkingCost)> + use<'s> {
        Nbor::ALL.iter().filter_map(move |&nbor| {
            self.get_nbor(pos, nbor).ok().map(|npos| {
                (
                    nbor,
                    npos,
                    WalkingCost::new(
                        nbor.distance(),
                        self.location
                            .get_first(npos, &self.accessibles)
                            .map_or_else(MoveCost::default, |floor| floor.move_cost),
                    ),
                )
            })
        })
    }

    /// Nbor from the first pos
    pub(crate) fn nbor(&self, one: Pos, other: Pos) -> Option<Nbor> {
        self.nbors_if(one, move |npos| npos == other)
            .next()
            .map(|(nbor, ..)| nbor)
    }

    fn nbors_if<F>(
        &'s self,
        pos: Pos,
        acceptable: F,
    ) -> impl Iterator<Item = (Nbor, Pos, WalkingCost)> + use<'s, F>
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
    ) -> impl Iterator<Item = (Nbor, Pos, Duration)> + use<'s> {
        self.nbors_if(pos, move |nbor| {
            (pos.level == Level::ZERO || self.location.all(pos).next().is_some()) && {
                let at_destination = Some(nbor) == destination;
                match intelligence {
                    Intelligence::Smart => {
                        self.is_accessible(nbor)
                            && (at_destination || self.find_obstacle(nbor).is_none())
                    }
                    Intelligence::Dumb => at_destination || !self.is_opaque(nbor),
                }
            }
        })
        .map(move |(nbor, npos, walking_cost)| (nbor, npos, walking_cost.duration(speed)))
    }

    pub(crate) fn directions_for_item_handling(
        &'s self,
        pos: Pos,
    ) -> impl Iterator<Item = (HorizontalDirection, Pos)> + use<'s> {
        self.nbors(pos).filter_map(|(nbor, npos, _)| {
            HorizontalDirection::try_from(nbor)
                .ok()
                .map(|direction| (direction, npos))
        })
    }

    pub(crate) fn nbors_for_exploring(
        &'s self,
        pos: Pos,
        instruction: &'s QueuedInstruction,
    ) -> impl Iterator<Item = Nbor> + use<'s> {
        self.nbors_if(pos, move |nbor| match instruction {
            QueuedInstruction::Attack => nbor != pos && self.find_character(nbor).is_some(),
            QueuedInstruction::Smash => self.find_smashable(nbor).is_some(),
            QueuedInstruction::Pulp => self.find_pulpable(nbor).is_some(),
            QueuedInstruction::Close => self.find_closeable(nbor).is_some(),
            QueuedInstruction::StartCraft { .. } => {
                self.is_accessible(nbor) && !self.is_water(nbor)
            }
            _ => panic!("unexpected instruction {instruction:?}"),
        })
        .map(move |(nbor, _npos, _distance)| nbor)
    }

    /// `WalkingCost` without regard for obstacles or stairs, unless they are nbors
    pub(crate) fn walking_cost(&self, from: Pos, to: Pos) -> WalkingCost {
        let dx = u64::from(from.x.abs_diff(to.x));
        let dz = u64::from(from.z.abs_diff(to.z));
        let diagonal = dx.min(dz);
        let adjacent = dx.max(dz) - diagonal;

        let dy = (to.level - from.level).h;
        let up = dy.max(0) as u64;
        let down = (-dy).max(0) as u64;

        let move_cost = if diagonal + adjacent + up + down == 1 {
            // nbors, the precise value matters in some cases
            // Dumb creatures may try to use paths that do not have a floor.
            self.location
                .get_first(to, &self.accessibles)
                .map_or_else(MoveCost::default, |floor| {
                    floor
                        .move_cost
                        .adjust(self.location.get_first(to, &self.hurdles).map(|h| h.0))
                })
        } else {
            // estimate
            MoveCost::default()
        };

        [
            (NborDistance::Up, up),
            (NborDistance::Adjacent, adjacent),
            (NborDistance::Diagonal, diagonal),
            (NborDistance::Down, down),
        ]
        .into_iter()
        .map(|(nd, amount)| WalkingCost::new(nd, move_cost) * amount)
        .sum()
    }

    pub(crate) fn path<F>(
        &self,
        from: Pos,
        to: Pos,
        intelligence: Intelligence,
        seen: F,
        speed: Speed,
        stay_duration: Duration,
    ) -> Option<MovementPath>
    where
        F: Fn(Pos) -> bool,
    {
        if to == from {
            return None;
        }

        let nbors_fn = |pos: &Pos| {
            self.nbors_for_moving(*pos, Some(to), intelligence, speed)
                .filter(|(_, pos, _)| seen(*pos))
                .map(|(_, pos, cost)| (pos, cost))
        };
        let estimated_duration_fn = |&pos: &Pos| {
            self.walking_cost(from, pos)
                .duration(speed)
                .max(stay_duration)
                + self.walking_cost(pos, to).duration(speed)
        };

        //trace!("dumb? {dumb:?} @{from:?}");
        if intelligence == Intelligence::Smart && seen(to) {
            MovementPath::plan(from, nbors_fn, estimated_duration_fn, to)
        } else {
            MovementPath::improvize(nbors_fn(&from), estimated_duration_fn, from, to)
        }
    }

    pub(crate) fn collide(&self, from: Pos, to: Pos, controlled: bool) -> Collision {
        assert_ne!(from, to, "Collisions require movement");
        assert!(
            self.nbor(from, to).is_some(),
            "Collisions require neighbor positions"
        );

        match to.level.cmp(&from.level) {
            Ordering::Greater | Ordering::Less => {
                if self.find_openable(to).is_some() {
                    unimplemented!();
                } else if controlled {
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
                if let Some((openable, _)) = if controlled {
                    self.find_openable(to)
                } else {
                    None
                } {
                    Collision::Opened(openable)
                } else if let Some(obstacle) = self.find_obstacle(to) {
                    Collision::Blocked(obstacle)
                } else if self.is_accessible(to) {
                    Collision::Pass
                } else if controlled {
                    Collision::Ledged
                } else {
                    unimplemented!();
                }
            }
        }
    }

    pub(crate) fn magic_stairs_up(&self) -> impl Iterator<Item = Pos> + use<'_> {
        self.stairs_up
            .iter()
            .filter(|pos| {
                pos.raw_nbor(Nbor::Up)
                    .is_some_and(|down| !self.location.has_stairs_down(down, &self.stairs_down))
            })
            .copied()
    }

    pub(crate) fn magic_stairs_down(&self) -> impl Iterator<Item = Pos> + use<'_> {
        self.stairs_down
            .iter()
            .filter(|pos| {
                pos.raw_nbor(Nbor::Down)
                    .is_some_and(|down| !self.location.has_stairs_up(down, &self.stairs_up))
            })
            .copied()
    }
}

#[derive(Debug)]
pub(crate) struct MovementPath {
    pub(crate) first: Pos,
    pub(crate) duration: Duration,
    pub(crate) destination: Pos,
}

impl MovementPath {
    pub(crate) fn plan<FN, IN, FH>(
        from: Pos,
        successors: FN,
        heuristic: FH,
        destination: Pos,
    ) -> Option<Self>
    where
        FN: FnMut(&Pos) -> IN,
        IN: Iterator<Item = (Pos, Duration)>,
        FH: FnMut(&Pos) -> Duration,
    {
        if let Some((mut steps, duration)) =
            astar(&from, successors, heuristic, |&pos| pos == destination)
        {
            assert!(
                2 <= steps.len(),
                "Movement steps should contain both the start and end point"
            );
            assert_eq!(
                steps.remove(0),
                from,
                "The first step should match the starting point"
            );
            Some(Self {
                first: *steps
                    .first()
                    .expect("The last step should always be present"),
                duration,
                destination,
            })
        } else {
            None
        }
    }

    pub(crate) fn improvize<I, FH>(
        nbors: I,
        mut heuristic: FH,
        from: Pos,
        destination: Pos,
    ) -> Option<Self>
    where
        I: Iterator<Item = (Pos, Duration)>,
        FH: FnMut(&Pos) -> Duration,
    {
        let paths = nbors
            .map(|(first, _)| Self {
                first,
                duration: heuristic(&first),
                destination,
            })
            //.inspect(|path| {
            //    trace!(
            //        "MovementPath::improvize {from:?} by {:?} to {destination:?} @ {:?}",
            //        path.first, path.duration
            //    );
            //})
            .collect::<Vec<_>>();

        let shortest = paths
            .iter()
            .min_by_key(|path| (path.first == from, path.duration))?
            .duration;
        let offset = (destination - from).vec3();
        let all_best = paths
            .into_iter()
            .filter(|path| path.duration == shortest)
            .map(|path| {
                if path.first == destination {
                    (1.0, path)
                } else {
                    let match_with_straight_line = offset
                        .angle_between((destination - path.first).vec3())
                        .cos();
                    (match_with_straight_line, path)
                }
            })
            .scan(0.0, |total, (score, path)| {
                *total += score;
                Some((score, path, *total))
            })
            .inspect(|(score, path, total)| {
                debug!(
                    "{from:?}->{:?}->{destination:?}  {score:?} / {total:?}",
                    path.first
                );
            })
            .collect::<Vec<_>>();
        let pick = all_best.last().expect("Not empty").2 * fastrand::f32();
        all_best
            .into_iter()
            .find(|(_, _, total)| pick <= *total)
            .map(|(_, path, _)| path)
    }
}
