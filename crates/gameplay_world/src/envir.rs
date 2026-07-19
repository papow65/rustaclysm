use crate::NoStairs;
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Entity, Query, Res, With, Without, warn};
use cdda_json_files::MoveCost;
use gameplay_common::WalkingCost;
use gameplay_item::{Amount, Item, ItemItem};
use gameplay_location::{
    HorizontalDirection, Level, LevelOffset, LocationCache, Nbor, NborDistance, Pos, PosOffset,
    StairsDown, StairsUp, Zone, ZoneLevel,
};
use gameplay_object::{Closeable, Corpse, Hurdle, Life, Obstacle, Opaque, Openable};
use gameplay_object::{ObjectName, StandardIntegrity};
use gameplay_terrain::{Accessible, OpaqueFloor};
use std::cmp::Ordering;

#[must_use]
pub enum Collision<'a> {
    Pass,
    //Fall(Pos), // todo
    Blocked(&'a ObjectName),
    Ledged,
    Opened(Entity),
}

#[derive(SystemParam)]
pub struct Envir<'w, 's> {
    location: Res<'w, LocationCache>,
    accessibles: Query<'w, 's, &'static Accessible>,
    hurdles: Query<'w, 's, &'static Hurdle>,
    openables: Query<'w, 's, (Entity, &'static ObjectName), With<Openable>>,
    closeables: Query<'w, 's, (Entity, &'static ObjectName), With<Closeable>>,
    stairs_up: Query<'w, 's, &'static Pos, With<StairsUp>>,
    stairs_down: Query<'w, 's, &'static Pos, With<StairsDown>>,
    terrain: Query<'w, 's, &'static ObjectName, (Without<Life>, Without<Corpse>, Without<Amount>)>,
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

    #[must_use]
    pub fn exists(&self, pos: Pos) -> bool {
        pos.level == Level::ZERO || self.location.all(pos).next().is_some()
    }

    #[must_use]
    pub fn is_accessible(&self, pos: Pos) -> bool {
        self.location.any(pos, &self.accessibles)
    }

    #[must_use]
    pub fn is_water(&self, pos: Pos) -> bool {
        self.location
            .get_first(pos, &self.accessibles)
            .is_some_and(|floor| floor.water)
    }

    #[must_use]
    pub fn stairs_up_to(&self, from: Pos) -> Option<Pos> {
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

    #[must_use]
    pub fn stairs_down_to(&self, from: Pos) -> Option<Pos> {
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

    #[must_use]
    pub fn find_accessibles(&self, pos: Pos) -> Option<&Accessible> {
        self.location.get_first(pos, &self.accessibles)
    }

    #[must_use]
    pub fn find_hurdles(&self, pos: Pos) -> Option<&Hurdle> {
        self.location.get_first(pos, &self.hurdles)
    }

    #[must_use]
    pub fn find_openable(&self, pos: Pos) -> Option<(Entity, &ObjectName)> {
        self.location.get_first(pos, &self.openables)
    }

    #[must_use]
    pub fn find_closeable(&self, pos: Pos) -> Option<(Entity, &ObjectName)> {
        self.location.get_first(pos, &self.closeables)
    }

    #[must_use]
    pub fn find_terrain(&self, pos: Pos) -> Option<&ObjectName> {
        self.location.get_first(pos, &self.terrain)
    }

    #[must_use]
    pub fn find_obstacle(&self, pos: Pos) -> Option<&ObjectName> {
        self.location.get_first(pos, &self.obstacles)
    }

    #[must_use]
    pub fn is_opaque(&self, pos: Pos) -> bool {
        self.location.any(pos, &self.opaques)
    }

    #[must_use]
    pub fn has_opaque_floor(&self, pos: Pos) -> bool {
        self.location.any(pos, &self.opaque_floors)
    }

    #[must_use]
    pub fn find_character(&self, pos: Pos) -> Option<(Entity, &ObjectName)> {
        self.location.get_first(pos, &self.characters)
    }

    #[must_use]
    pub fn find_smashable(&self, pos: Pos) -> Option<Entity> {
        self.location.get_first(pos, &self.smashables)
    }

    #[must_use]
    pub fn find_pulpable(&self, pos: Pos) -> Option<Entity> {
        self.location.get_first(pos, &self.pulpables)
    }

    #[must_use]
    pub fn find_item(&self, pos: Pos) -> Option<ItemItem<'_, '_>> {
        self.location.get_first(pos, &self.items)
    }

    pub fn all_items(&self, pos: Pos) -> impl Iterator<Item = ItemItem<'_, '_>> {
        self.location
            .all(pos)
            .flat_map(|&entity| self.items.get(entity))
    }

    // helper methods

    /// In case of vertical nbors: Follow stairs, even when they do not go staight up or down. Without stairs, see the raw position below/above, unless that contains a stair to somewhere else.
    #[must_use]
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
    ///
    /// # Errors
    /// On vertical nbors when there are no stairs
    pub fn get_nbor(&self, from: Pos, nbor: Nbor) -> Result<Pos, NoStairs> {
        match nbor {
            Nbor::Up => self.stairs_up_to(from).ok_or(NoStairs::Up),
            Nbor::Down => self.stairs_down_to(from).ok_or(NoStairs::Down),
            Nbor::Horizontal(horizontal_direction) => {
                // No stairs
                Ok(from.horizontal_nbor(horizontal_direction))
            }
        }
    }

    /// Follow stairs, even when they do not go staight up or down.
    #[must_use]
    pub fn to_nbor(&self, from: Pos, to: Pos) -> Option<Nbor> {
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

    pub fn nbors(&'s self, pos: Pos) -> impl Iterator<Item = (Nbor, Pos, WalkingCost)> + use<'s> {
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
    #[must_use]
    pub fn nbor(&self, one: Pos, other: Pos) -> Option<Nbor> {
        self.nbors_if(one, move |npos| npos == other)
            .next()
            .map(|(nbor, ..)| nbor)
    }

    pub fn nbors_if<F>(
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

    pub fn directions_for_item_handling(
        &'s self,
        pos: Pos,
    ) -> impl Iterator<Item = (HorizontalDirection, Pos)> + use<'s> {
        self.nbors(pos).filter_map(|(nbor, npos, _)| {
            HorizontalDirection::try_from(nbor)
                .ok()
                .map(|direction| (direction, npos))
        })
    }

    pub fn nbors_to_attack(&'s self, pos: Pos) -> impl Iterator<Item = Nbor> + use<'s> {
        self.nbors_if(pos, move |nbor| {
            nbor != pos && self.find_character(nbor).is_some()
        })
        .map(move |(nbor, _npos, _distance)| nbor)
    }

    pub fn nbors_to_smash(&'s self, pos: Pos) -> impl Iterator<Item = Nbor> + use<'s> {
        self.nbors_if(pos, move |nbor| self.find_smashable(nbor).is_some())
            .map(move |(nbor, _npos, _distance)| nbor)
    }

    pub fn directions_to_pulp(
        &'s self,
        pos: Pos,
    ) -> impl Iterator<Item = HorizontalDirection> + use<'s> {
        self.nbors_if(pos, move |nbor| self.find_pulpable(nbor).is_some())
            .filter_map(move |(nbor, _npos, _distance)| HorizontalDirection::try_from(nbor).ok())
    }

    pub fn directions_to_close(
        &'s self,
        pos: Pos,
    ) -> impl Iterator<Item = HorizontalDirection> + use<'s> {
        self.nbors_if(pos, move |nbor| self.find_closeable(nbor).is_some())
            .filter_map(move |(nbor, _npos, _distance)| HorizontalDirection::try_from(nbor).ok())
    }

    pub fn directions_to_craft(
        &'s self,
        pos: Pos,
    ) -> impl Iterator<Item = HorizontalDirection> + use<'s> {
        self.nbors_if(pos, move |nbor| {
            self.is_accessible(nbor) && !self.is_water(nbor)
        })
        .filter_map(move |(nbor, _npos, _distance)| HorizontalDirection::try_from(nbor).ok())
    }

    /// With regard for obstacles and stairs
    pub fn nbor_walking_cost(&self, from: Pos, nbor: Nbor) -> Result<WalkingCost, NoStairs> {
        let to = self.get_nbor(from, nbor)?;

        let move_cost = self
            .find_accessibles(to)
            .map_or_else(MoveCost::default, |floor| {
                floor.move_cost.adjust(self.find_hurdles(to).map(|h| h.0))
            });

        Ok(self.walking_cost(from, to, move_cost))
    }

    /// Without regard for obstacles or stairs
    pub fn estimated_walking_cost(&self, from: Pos, to: Pos) -> WalkingCost {
        self.walking_cost(from, to, MoveCost::default())
    }

    fn walking_cost(&self, from: Pos, to: Pos, move_cost: MoveCost) -> WalkingCost {
        let dx = u64::from(from.x.abs_diff(to.x));
        let dz = u64::from(from.z.abs_diff(to.z));
        let diagonal = dx.min(dz);
        let adjacent = dx.max(dz) - diagonal;

        let dy = (to.level - from.level).h;
        let up = dy.max(0) as u64;
        let down = (-dy).max(0) as u64;

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

    pub fn collide(&self, from: Pos, to: Pos, controlled: bool) -> Collision<'_> {
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

    pub fn magic_stairs_up(&self) -> impl Iterator<Item = Pos> + use<'_> {
        self.stairs_up
            .iter()
            .filter(|pos| {
                pos.raw_nbor(Nbor::Up)
                    .is_some_and(|down| !self.location.has_stairs_down(down, &self.stairs_down))
            })
            .copied()
    }

    pub fn magic_stairs_down(&self) -> impl Iterator<Item = Pos> + use<'_> {
        self.stairs_down
            .iter()
            .filter(|pos| {
                pos.raw_nbor(Nbor::Down)
                    .is_some_and(|down| !self.location.has_stairs_up(down, &self.stairs_up))
            })
            .copied()
    }
}
