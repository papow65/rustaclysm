use crate::prelude::*;
use bevy::{ecs::system::SystemParam, prelude::*, utils::HashMap};
use pathfinding::prelude::astar;
use std::{cell::RefCell, cmp::Ordering, iter::repeat};

pub(crate) enum Collision {
    Pass,
    //Fall(Pos), // todo
    Blocked(TextLabel),
    Ledged,
    Opened(Entity),
}

#[derive(SystemParam)]
pub(crate) struct Envir<'w, 's> {
    pub(crate) location: ResMut<'w, Location>,
    relative_segments: Res<'w, RelativeSegments>,
    accessibles: Query<'w, 's, &'static Accessible>,
    hurdles: Query<'w, 's, &'static Hurdle>,
    openables: Query<'w, 's, Entity, With<Openable>>,
    closeables: Query<'w, 's, Entity, With<Closeable>>,
    stairs_up: Query<'w, 's, &'static StairsUp>,
    stairs_down: Query<'w, 's, &'static StairsDown>,
    terrain: Query<'w, 's, &'static TextLabel, (Without<Health>, Without<Amount>)>,
    obstacles: Query<'w, 's, &'static TextLabel, With<Obstacle>>,
    opaques: Query<'w, 's, &'static TextLabel, With<Opaque>>,
    opaque_floors: Query<'w, 's, &'static OpaqueFloor>,
    characters: Query<'w, 's, (Entity, &'static TextLabel), With<Health>>,
    items: Query<'w, 's, Entity, With<Integrity>>,
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

    pub(crate) fn stairs_up_to(&self, pos: Pos) -> Option<Pos> {
        if self.location.has_stairs_up(pos, &self.stairs_up) {
            let zone_level_up = ZoneLevel::from(pos).offset(LevelOffset::UP).unwrap();

            for distance in 0..24i32 {
                for dx in -distance..=distance {
                    for dz in -distance..=distance {
                        if dx.abs().max(dz.abs()) == distance {
                            let test_up = pos
                                .offset(PosOffset {
                                    x: dx,
                                    level: LevelOffset::UP,
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

            eprintln!("No matching stairs up found");
            pos.offset(PosOffset {
                x: 0,
                level: LevelOffset::UP,
                z: 0,
            })
        } else {
            None
        }
    }

    pub(crate) fn stairs_down_to(&self, pos: Pos) -> Option<Pos> {
        if self.location.has_stairs_down(pos, &self.stairs_down) {
            let zone_level_down = ZoneLevel::from(pos).offset(LevelOffset::DOWN).unwrap();

            // fast approach in most cases, otherwise fast enough
            for distance in 0..Zone::SIZE {
                for dx in -distance..=distance {
                    for dz in -distance..=distance {
                        if dx.abs().max(dz.abs()) == distance {
                            let test_down = pos
                                .offset(PosOffset {
                                    x: dx,
                                    level: LevelOffset::DOWN,
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

            eprintln!("No matching stairs down found");
            pos.offset(PosOffset {
                x: 0,
                level: LevelOffset::DOWN,
                z: 0,
            })
        } else {
            None
        }
    }

    pub(crate) fn find_openable(&self, pos: Pos) -> Option<Entity> {
        self.location.get_first(pos, &self.openables)
    }

    pub(crate) fn find_closeable(&self, pos: Pos) -> Option<Entity> {
        self.location.get_first(pos, &self.closeables)
    }

    pub(crate) fn find_terrain(&self, pos: Pos) -> Option<&TextLabel> {
        self.location.get_first(pos, &self.terrain)
    }

    pub(crate) fn find_obstacle(&self, pos: Pos) -> Option<&TextLabel> {
        self.location.get_first(pos, &self.obstacles)
    }

    fn is_opaque(&self, pos: Pos) -> bool {
        self.location.any(pos, &self.opaques)
    }

    pub(crate) fn has_opaque_floor(&self, pos: Pos) -> bool {
        self.location.any(pos, &self.opaque_floors)
    }

    pub(crate) fn find_character(&self, pos: Pos) -> Option<(Entity, &TextLabel)> {
        self.location.get_first(pos, &self.characters)
    }

    pub(crate) fn find_item(&self, pos: Pos) -> Option<Entity> {
        self.location.get_first(pos, &self.items)
    }

    // helper methods

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

    fn nbors(&'s self, pos: Pos) -> impl Iterator<Item = (Nbor, Pos, WalkingCost)> + 's {
        Nbor::ALL.iter().filter_map(move |nbor| {
            self.get_nbor(pos, nbor).ok().map(|npos| {
                (
                    nbor.clone(),
                    npos,
                    WalkingCost::new(
                        &nbor.distance(),
                        self.location
                            .get_first(npos, &self.accessibles)
                            .map_or_else(MoveCost::default, |floor| floor.move_cost),
                    ),
                )
            })
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
    ) -> impl Iterator<Item = (Nbor, Pos, WalkingCost)> + 's
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
        speed: BaseSpeed,
    ) -> impl Iterator<Item = (Pos, Milliseconds)> + 's {
        self.nbors_if(pos, move |nbor| {
            (pos.level == Level::ZERO || !self.location.all(pos).is_empty()) && {
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
        .map(move |(_nbor, npos, walking_cost)| (npos, walking_cost.duration(speed)))
    }

    pub(crate) fn nbors_for_exploring(
        &'s self,
        pos: Pos,
        instruction: QueuedInstruction,
    ) -> impl Iterator<Item = Pos> + 's {
        self.nbors_if(pos, move |nbor| match instruction {
            QueuedInstruction::Attack => nbor != pos && self.find_character(nbor).is_some(),
            QueuedInstruction::Smash => self.find_item(nbor).is_some(),
            QueuedInstruction::Close => self.find_closeable(nbor).is_some(),
            _ => panic!("unexpected instruction {instruction:?}"),
        })
        .map(move |(_nbor, npos, _distance)| npos)
    }

    /** `WalkingCost` without regard for obstacles or stairs, unless they are nbors */
    pub(crate) fn walking_cost(&self, from: Pos, to: Pos) -> WalkingCost {
        let dx = from.x.abs_diff(to.x) as usize;
        let dz = from.z.abs_diff(to.z) as usize;
        let diagonal = dx.min(dz);
        let adjacent = dx.max(dz) - diagonal;

        let dy = (to.level - from.level).h;
        let up = (dy as usize).min(0);
        let down = (-dy as usize).min(0);

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

        repeat(NborDistance::Up)
            .take(up)
            .chain(repeat(NborDistance::Adjacent).take(adjacent))
            .chain(repeat(NborDistance::Diagonal).take(diagonal))
            .chain(repeat(NborDistance::Down).take(down))
            .map(|nd| WalkingCost::new(&nd, move_cost))
            .reduce(|total, item| total + item)
            .unwrap_or_else(|| WalkingCost::new(&NborDistance::Zero, move_cost))
    }

    pub(crate) fn path(
        &self,
        from: Pos,
        to: Pos,
        intelligence: Intelligence,
        speed: BaseSpeed,
    ) -> Option<MovementPath> {
        if to == from {
            return None;
        }

        let nbors_fn = |pos: &Pos| self.nbors_for_moving(*pos, Some(to), intelligence, speed);
        let estimated_duration_fn = |&pos: &Pos| self.walking_cost(pos, to).duration(speed);

        //println!("dumb? {dumb:?} @{from:?}");
        match intelligence {
            Intelligence::Smart => MovementPath::plan(from, nbors_fn, estimated_duration_fn, to),
            Intelligence::Dumb => {
                MovementPath::improvize(nbors_fn(&from), estimated_duration_fn, to)
            }
        }
    }

    pub(crate) fn collide(&self, from: Pos, to: Pos, controlled: bool) -> Collision {
        assert_ne!(from, to);
        assert!(self.are_nbors(from, to));

        match to.level.cmp(&from.level) {
            Ordering::Greater | Ordering::Less => {
                if self.find_openable(to).is_some() {
                    unimplemented!();
                } else if controlled {
                    if let Some(obstacle) = self.find_obstacle(to) {
                        Collision::Blocked(obstacle.clone())
                    } else {
                        Collision::Pass
                    }
                } else {
                    unimplemented!();
                }
            }
            Ordering::Equal => {
                if let Some(openable) = if controlled {
                    self.find_openable(to)
                } else {
                    None
                } {
                    Collision::Opened(openable)
                } else if let Some(obstacle) = self.find_obstacle(to) {
                    Collision::Blocked(obstacle.clone())
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

    pub(crate) fn currently_visible(&self, from: Pos) -> CurrentlyVisible {
        CurrentlyVisible {
            envir: self,
            from,
            opaque_cache: RefCell::default(),
            down_cache: RefCell::default(),
            visible_cache: RefCell::default(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct MovementPath {
    pub(crate) first: Pos,
    pub(crate) duration: Milliseconds,
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
        IN: Iterator<Item = (Pos, Milliseconds)>,
        FH: FnMut(&Pos) -> Milliseconds,
    {
        if let Some((mut steps, duration)) =
            astar(&from, successors, heuristic, |&pos| pos == destination)
        {
            assert!(!steps.is_empty());
            assert_eq!(steps.remove(0), from);
            assert!(!steps.is_empty());
            Some(Self {
                first: *steps.first().unwrap(),
                duration,
                destination,
            })
        } else {
            None
        }
    }

    pub(crate) fn improvize<I, FH>(nbors: I, mut heuristic: FH, destination: Pos) -> Option<Self>
    where
        I: Iterator<Item = (Pos, Milliseconds)>,
        FH: FnMut(&Pos) -> Milliseconds,
    {
        nbors
            .map(|(first, duration)| Self {
                first,
                duration: duration + heuristic(&first),
                destination,
            })
            .min_by_key(|path| path.duration)
    }
}

pub(crate) struct CurrentlyVisible<'a> {
    envir: &'a Envir<'a, 'a>,
    from: Pos,
    opaque_cache: RefCell<HashMap<PosOffset, bool>>, // is opaque
    down_cache: RefCell<HashMap<PosOffset, bool>>,   // can see down
    visible_cache: RefCell<HashMap<PosOffset, Visible>>,
}

impl<'a> CurrentlyVisible<'a> {
    pub(crate) fn can_see(&self, to: Pos, accessible: Option<&Accessible>) -> Visible {
        let to = if accessible.is_some() && self.from.level < to.level {
            // seen from below?
            Pos {
                x: to.x,
                level: Level { h: to.level.h - 1 },
                z: to.z,
            }
        } else {
            // seen from above?
            to
        };

        if MIN_INVISIBLE_DISTANCE <= self.from.x.abs_diff(to.x)
            || MIN_INVISIBLE_DISTANCE <= self.from.z.abs_diff(to.z)
            || (accessible.is_some() && self.envir.is_opaque(to))
        {
            Visible::Unseen
        } else {
            self.can_see_relative(to - self.from)
        }
    }

    pub(crate) fn can_see_relative(&self, relative_to: PosOffset) -> Visible {
        if let Some(visible) = self.visible_cache.borrow().get(&relative_to) {
            return visible.clone();
        }

        let Some(relative_segment) = self.envir.relative_segments.segments.get(&relative_to) else {
            return self.remember_visible(relative_to, Visible::Unseen);
        };

        if relative_segment
            .preceding
            .map(|preceding| self.can_see_relative(preceding))
            == Some(Visible::Unseen)
        {
            return self.remember_visible(relative_to, Visible::Unseen);
        }

        if relative_segment
            .segment
            .iter()
            .any(|offset| self.is_opaque(*offset))
        {
            return self.remember_visible(relative_to, Visible::Unseen);
        }

        let visible = if relative_segment
            .down_pairs
            .iter()
            .all(|(offset_a, offset_b)| {
                self.can_look_down(*offset_a) || self.can_look_down(*offset_b)
            }) {
            Visible::Seen
        } else {
            Visible::Unseen
        };
        self.remember_visible(relative_to, visible)
    }

    fn is_opaque(&self, offset: PosOffset) -> bool {
        *self
            .opaque_cache
            .borrow_mut()
            .entry(offset)
            .or_insert_with(|| {
                let to = self.from.offset(offset).unwrap();
                self.envir.is_opaque(to)
                    || (to.level < Level::ZERO && self.envir.find_terrain(to).is_none())
            })
    }

    fn can_look_down(&self, offset: PosOffset) -> bool {
        *self
            .down_cache
            .borrow_mut()
            .entry(offset)
            .or_insert_with(|| {
                let to = self.from.offset(offset).unwrap();
                !self.envir.has_opaque_floor(to)
                    && (Level::ZERO <= to.level || self.envir.is_accessible(to))
            })
    }

    fn remember_visible(&self, relative_to: PosOffset, visible: Visible) -> Visible {
        self.visible_cache
            .borrow_mut()
            .insert(relative_to, visible.clone());
        visible
    }
}