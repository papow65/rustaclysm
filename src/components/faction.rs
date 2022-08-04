use crate::prelude::*;
use bevy::ecs::component::Component;
use rand::prelude::*;
use rand::seq::SliceRandom;
use std::iter::once;

pub const SAFETY: Milliseconds = Milliseconds(10000);

#[derive(Copy, Clone, PartialEq)]
pub enum Intelligence {
    Dumb,
    Smart,
}

pub struct Strategy {
    pub intent: Intent,
    pub action: Action,
}

#[derive(Copy, Clone, Debug)]
pub enum Intent {
    Attack,
    Flee,
    Wander,
    Wait,
}

#[derive(Component, Debug)]
pub enum Faction {
    Human,
    Zombie,
    Animal,
}

impl Faction {
    fn equals(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    pub fn is_aggressive(&self, health: &Health) -> bool {
        match self {
            Self::Human => health.relative_damage() < Partial::from_u8(128),
            _ => true,
        }
    }

    pub fn dislikes(&self, other: &Self) -> bool {
        !self.equals(other)
    }

    pub fn can_fear(&self) -> bool {
        !self.equals(&Self::Zombie)
    }

    pub fn wanders(&self) -> bool {
        !self.equals(&Self::Human)
    }

    pub const fn intelligence(&self) -> Intelligence {
        match self {
            Self::Zombie => Intelligence::Dumb,
            _ => Intelligence::Smart,
        }
    }

    fn intents(&self, health: &Health) -> impl Iterator<Item = Intent> {
        once(self.is_aggressive(health).then(|| Intent::Attack))
            .chain(once(self.can_fear().then(|| Intent::Flee)))
            .chain(once(self.wanders().then(|| Intent::Wander)))
            .flatten()
            .chain(once(Intent::Wait))
    }

    pub fn attack(
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

    pub fn flee(
        &self,
        envir: &Envir,
        start_pos: Pos,
        speed: Speed,
        enemies: &[Pos],
    ) -> Option<Action> {
        if enemies.is_empty() {
            return None;
        }

        let penalty = |pos: &Pos| {
            let safe_for = enemies
                .iter()
                .filter_map(|enemy_pos| envir.path(*pos, *enemy_pos, self.intelligence(), speed))
                .map(|path| path.duration)
                .chain(std::iter::once(SAFETY))
                .min()
                .unwrap();

            (1000.0 / safe_for.0 as f32 - 1000.0 / SAFETY.0 as f32) as i64
        };
        Some(Action::step_or_stay(
            envir.find_best(start_pos, speed, penalty),
        ))
    }

    pub fn wander(&self, envir: &Envir, start_pos: Pos, speed: Speed) -> Option<Action> {
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

    pub fn behave<'f>(
        &self,
        envir: &Envir,
        start_pos: Pos,
        speed: Speed,
        health: &Health,
        factions: &[(Pos, &'f Self)],
    ) -> Strategy {
        let enemies = factions
            .iter()
            .filter(|(_, other_faction)| self.dislikes(other_faction))
            .map(|(enemy_pos, _)| enemy_pos)
            .copied()
            .filter(|enemy_pos| envir.can_see(start_pos, *enemy_pos) == PlayerVisible::Seen)
            .collect::<Vec<Pos>>();
        println!("{self:?} can see {:?} enemies", enemies.len());

        self.intents(health)
            .find_map(|intent| self.attempt(intent, envir, start_pos, speed, factions, &enemies))
            .unwrap()
    }
}
