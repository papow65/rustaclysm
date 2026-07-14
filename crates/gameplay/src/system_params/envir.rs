use crate::{Closeable, Corpse, Health, Hurdle, Life, NoStairs, Obstacle, Opaque, Openable};
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Entity, Query, Res, With, Without, warn};
use cdda_json_files::MoveCost;
use gameplay_common::{ObjectName, StandardIntegrity, WalkingCost};
use gameplay_item::{Amount, Item, ItemItem};
use gameplay_location::{
    HorizontalDirection, Level, LevelOffset, LocationCache, Nbor, Pos, PosOffset, StairsDown,
    StairsUp, Zone, ZoneLevel,
};
use gameplay_terrain::{Accessible, OpaqueFloor};
use std::cmp::Ordering;

pub(crate) enum Collision<'a> {
    Pass,
    //Fall(Pos), // todo
    Blocked(&'a ObjectName),
    Ledged,
    Opened(Entity),
}

#[derive(SystemParam)]
pub(crate) struct Envir<'w, 's> {
    location: Res<'w, LocationCache>,
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

    pub(crate) fn exists(&self, pos: Pos) -> bool {
        pos.level == Level::ZERO || self.location.all(pos).next().is_some()
    }

    pub(crate) fn is_accessible(&self, pos: Pos) -> bool {
        self.location.any(pos, &self.accessibles)
    }

    pub(crate) fn is_water(&self, pos: Pos) -> bool {
        self.location
            .get_first(pos, &self.accessibles)
            .is_some_and(|floor| floor.water)
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

    pub(crate) fn find_accessibles(&self, pos: Pos) -> Option<&Accessible> {
        self.location.get_first(pos, &self.accessibles)
    }

    pub(crate) fn find_hurdles(&self, pos: Pos) -> Option<&Hurdle> {
        self.location.get_first(pos, &self.hurdles)
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

    pub(crate) fn find_item(&self, pos: Pos) -> Option<ItemItem<'_, '_>> {
        self.location.get_first(pos, &self.items)
    }

    pub(crate) fn all_items(&self, pos: Pos) -> impl Iterator<Item = ItemItem<'_, '_>> {
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
    pub(crate) fn get_nbor(&self, from: Pos, nbor: Nbor) -> Result<Pos, NoStairs> {
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

    pub(crate) fn nbors(
        &'s self,
        pos: Pos,
    ) -> impl Iterator<Item = (Nbor, Pos, WalkingCost)> + use<'s> {
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

    pub(crate) fn nbors_if<F>(
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

    pub(crate) fn collide(&self, from: Pos, to: Pos, controlled: bool) -> Collision<'_> {
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
