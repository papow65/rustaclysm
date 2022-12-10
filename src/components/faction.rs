use crate::prelude::*;
use bevy::ecs::component::Component;
use pathfinding::prelude::{build_path, dijkstra_all};
use rand::prelude::*;
use rand::seq::SliceRandom;
use std::iter::once;

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Intelligence {
    Dumb,
    Smart,
}

pub(crate) struct Strategy {
    pub(crate) intent: Intent,
    pub(crate) action: Action,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum Intent {
    Attack,
    Flee,
    Wander,
    Wait,
}

#[derive(Component, Debug)]
pub(crate) enum Faction {
    Human,
    Zombie,
    Animal,
}

impl Faction {
    fn equals(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    pub(crate) fn is_aggressive(&self, health: &Health) -> bool {
        match self {
            Self::Human => health.relative_damage() < Partial::from_u8(128),
            _ => true,
        }
    }

    pub(crate) fn dislikes(&self, other: &Self) -> bool {
        !self.equals(other)
    }

    pub(crate) fn can_fear(&self) -> bool {
        !self.equals(&Self::Zombie)
    }

    pub(crate) fn wanders(&self) -> bool {
        !self.equals(&Self::Human)
    }

    pub(crate) const fn intelligence(&self) -> Intelligence {
        match self {
            Self::Zombie => Intelligence::Dumb,
            _ => Intelligence::Smart,
        }
    }

    fn intents(&self, health: &Health) -> impl Iterator<Item = Intent> {
        once(self.is_aggressive(health).then_some(Intent::Attack))
            .chain(once(self.can_fear().then_some(Intent::Flee)))
            .chain(once(self.wanders().then_some(Intent::Wander)))
            .flatten()
            .chain(once(Intent::Wait))
    }

    pub(crate) fn attack(
        &self,
        envir: &Envir,
        start_pos: Pos,
        speed: Speed,
        factions: &[(Pos, &Self)],
        enemies: &[Pos],
    ) -> Option<Action> {
        enemies
            .iter()
            .filter_map(|enemy_pos| envir.path(start_pos, *enemy_pos, self.intelligence(), speed))
            .min_by_key(|path| path.duration.0)
            .map(|path| {
                //println!("{:?}->{:?}", path.first, path.destination,);
                if path.first == path.destination {
                    Action::Attack { target: path.first }
                } else if envir.find_obstacle(path.first).is_some() {
                    if factions.iter().any(|(pos, _)| *pos == path.first) {
                        Action::Stay
                    } else {
                        Action::Smash { target: path.first }
                    }
                } else {
                    Action::Step { target: path.first }
                }
            })
    }

    pub(crate) fn flee(
        &self,
        envir: &Envir,
        start_pos: Pos,
        speed: Speed,
        enemies: &[Pos],
    ) -> Option<Action> {
        if enemies.is_empty() {
            return None;
        }

        let up_time: Milliseconds = WalkingDistance {
            horizontal: Millimeter(0),
            up: Millimeter::VERTICAL,
            down: Millimeter(0),
        } / speed;

        // Higher gives better results but is slower
        let planning_limit: u64 = 5;
        let min_time = Milliseconds((planning_limit - 1) * up_time.0); // included
        let max_time = Milliseconds(planning_limit * up_time.0); // not included

        let graph = dijkstra_all(&(start_pos, Milliseconds(0)), |(pos, prev_total_ms)| {
            envir
                .nbors_for_moving(*pos, None, self.intelligence(), speed)
                .filter_map(|(nbor, ms)| {
                    let total_ms = *prev_total_ms + ms;
                    if max_time < total_ms {
                        None
                    } else {
                        Some(((nbor, total_ms), Danger::new(&ms, nbor, enemies)))
                    }
                })
                .collect::<Vec<((Pos, Milliseconds), Danger)>>()
                .into_iter()
        });
        let safest_longtime_pos = graph
            .iter()
            .filter(|((_, ms), _)| min_time < *ms)
            .min_by_key(|((_, ms), (_, danger))| danger.average(ms))
            .expect("Non-empty graph")
            .0;
        let target = build_path(safest_longtime_pos, &graph)
            .get(1)
            .expect("First step (after current position)")
            .0;
        Some(if target == start_pos {
            Action::Stay
        } else {
            Action::Step { target }
        })
    }

    pub(crate) fn wander(&self, envir: &Envir, start_pos: Pos, speed: Speed) -> Option<Action> {
        let mut random = rand::thread_rng();

        if random.gen::<f32>() < 0.3 {
            envir
                .nbors_for_moving(start_pos, None, self.intelligence(), speed)
                .map(|(pos, _)| pos)
                .collect::<Vec<Pos>>()
                .choose(&mut random)
                .map(|&pos| {
                    if envir.find_character(pos).is_some() {
                        Action::Attack { target: pos }
                    } else if envir.find_item(pos).is_some() {
                        Action::Smash { target: pos }
                    } else {
                        Action::Step { target: pos }
                    }
                })
        } else {
            None
        }
    }

    fn attempt(
        &self,
        intent: Intent,
        envir: &Envir,
        start_pos: Pos,
        speed: Speed,
        factions: &[(Pos, &Self)],
        enemies: &[Pos],
    ) -> Option<Strategy> {
        match intent {
            Intent::Attack => self.attack(envir, start_pos, speed, factions, enemies),
            Intent::Flee => self.flee(envir, start_pos, speed, enemies),
            Intent::Wander => self.wander(envir, start_pos, speed),
            Intent::Wait => Some(Action::Stay),
        }
        .map(|action| Strategy { intent, action })
    }

    pub(crate) fn behave<'f>(
        &self,
        envir: &Envir,
        start_pos: Pos,
        speed: Speed,
        health: &Health,
        factions: &[(Pos, &'f Self)],
    ) -> Strategy {
        let currently_visible = envir.currently_visible(start_pos);

        let enemies = factions
            .iter()
            .filter(|(_, other_faction)| self.dislikes(other_faction))
            .map(|(enemy_pos, _)| enemy_pos)
            .copied()
            .filter(|enemy_pos| currently_visible.can_see(*enemy_pos) == Visible::Seen)
            .collect::<Vec<Pos>>();
        //println!("{self:?} can see {:?} enemies", enemies.len());

        self.intents(health)
            .find_map(|intent| self.attempt(intent, envir, start_pos, speed, factions, &enemies))
            .expect("Fallback intent")
    }
}
