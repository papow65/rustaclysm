use crate::prelude::*;
use bevy::prelude::Component;
use float_ord::FloatOrd;
use pathfinding::{
    num_traits::Zero,
    prelude::{build_path, dijkstra_all},
};
use rand::prelude::*;
use rand::seq::SliceRandom;
use std::ops::Add;

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Intelligence {
    Dumb,
    Smart,
}

pub(crate) struct Strategy {
    pub(crate) intent: Intent,
    pub(crate) action: Action,
    pub(crate) last_enemy: Option<LastEnemy>,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum Intent {
    Attack,
    Flee,
    Wander,
    Wait,
}

impl Intent {
    /** in order of consideration */
    const ALL: [Self; 4] = [Self::Attack, Self::Flee, Self::Wander, Self::Wait];
}

#[derive(Component, Debug, PartialEq)]
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

    fn consider(&self, intent: Intent, health: &Health) -> bool {
        match intent {
            Intent::Attack => self.is_aggressive(health),
            Intent::Flee => self.can_fear(),
            Intent::Wander => self.wanders(),
            Intent::Wait => true,
        }
    }

    pub(crate) fn attack(
        &self,
        envir: &Envir,
        factions: &[(Pos, &Self)],
        enemies: &[Pos],
        actor: &Actor,
    ) -> Option<(Action, LastEnemy)> {
        enemies
            .iter()
            .copied()
            .map(|pos| (false, pos))
            .chain(
                actor
                    .last_enemy
                    .iter()
                    .map(|last_enemy| (true, last_enemy.0)),
            )
            .filter_map(|(memory, enemy_pos)| {
                envir
                    .path(actor.pos, enemy_pos, self.intelligence(), actor.speed)
                    .map(|path| (memory, path))
            })
            .min_by_key(|(memory, path)| (*memory, path.duration.0))
            .and_then(|(_, path)| {
                let last_enemy = LastEnemy(path.destination);

                if path.first == path.destination {
                    if factions
                        .iter()
                        .filter(|(_, faction)| faction != &self)
                        .any(|(pos, _)| pos == &path.destination)
                    {
                        Some((Action::Attack { target: path.first }, last_enemy))
                    } else {
                        None
                    }
                } else if envir.find_obstacle(path.first).is_some() {
                    Some((
                        if factions.iter().any(|(pos, _)| *pos == path.first) {
                            Action::Stay
                        } else {
                            Action::Smash { target: path.first }
                        },
                        last_enemy,
                    ))
                } else {
                    Some((Action::Step { target: path.first }, last_enemy))
                }
            })
    }

    pub(crate) fn flee(&self, envir: &Envir, enemies: &[Pos], actor: &Actor) -> Option<Action> {
        if enemies.is_empty() {
            return None;
        }

        let up_time =
            WalkingCost::new(&NborDistance::Up, MoveCost::default()).duration(actor.speed);

        // Higher gives better results but is slower
        let planning_limit: u64 = 5;
        let min_time = Milliseconds((planning_limit - 1) * up_time.0); // included
        let max_time = Milliseconds(planning_limit * up_time.0); // not included

        let graph = dijkstra_all(&(actor.pos, Milliseconds(0)), |(pos, prev_total_ms)| {
            envir
                .nbors_for_moving(*pos, None, self.intelligence(), actor.speed)
                .filter_map(|(nbor, ms)| {
                    let total_ms = *prev_total_ms + ms;
                    if max_time < total_ms {
                        None
                    } else {
                        Some(((nbor, total_ms), Danger::new(envir, &ms, nbor, enemies)))
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
        Some(if target == actor.pos {
            Action::Stay
        } else {
            Action::Step { target }
        })
    }

    pub(crate) fn wander(
        &self,
        envir: &Envir,
        factions: &[(Pos, &Self)],
        actor: &Actor,
    ) -> Option<Action> {
        let mut random = rand::thread_rng();

        if random.gen::<f32>() < 0.3 {
            envir
                .nbors_for_moving(actor.pos, None, self.intelligence(), actor.speed)
                .filter(|(pos, _)| factions.iter().all(|(other_pos, _)| pos != other_pos))
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
        factions: &[(Pos, &Self)],
        enemies: &[Pos],
        actor: &Actor,
    ) -> Option<Strategy> {
        match intent {
            Intent::Attack => self
                .attack(envir, factions, enemies, actor)
                .map(|(action, last_enemy)| (action, Some(last_enemy))),
            Intent::Flee => self
                .flee(envir, enemies, actor)
                .map(|action| (action, None)),
            Intent::Wander => self
                .wander(envir, factions, actor)
                .map(|action| (action, None)),
            Intent::Wait => Some(Action::Stay).map(|action| (action, None)),
        }
        .filter(|(action, _)| match action {
            // prevent fish from acting on land
            Action::Step { target } | Action::Attack { target } | Action::Smash { target } => {
                actor.aquatic.is_none() || envir.is_water(*target)
            }
            _ => true,
        })
        .map(|(action, last_enemy)| Strategy {
            intent,
            action,
            last_enemy,
        })
    }

    pub(crate) fn strategize(
        &self,
        envir: &Envir,
        factions: &[(Pos, &Self)],
        actor: &Actor,
    ) -> Strategy {
        let currently_visible = envir.currently_visible(actor.pos);

        let enemies = factions
            .iter()
            .filter(|(_, other_faction)| self.dislikes(other_faction))
            .map(|(enemy_pos, _)| enemy_pos)
            .copied()
            .filter(|enemy_pos| actor.aquatic.is_none() || envir.is_water(*enemy_pos))
            .filter(|enemy_pos| currently_visible.can_see(*enemy_pos) == Visible::Seen)
            .collect::<Vec<Pos>>();
        //println!("{self:?} can see {:?} enemies", enemies.len());

        Intent::ALL
            .into_iter()
            .filter(|intent| self.consider(*intent, actor.health))
            .find_map(|intent| self.attempt(intent, envir, factions, &enemies, actor))
            .expect("Fallback intent")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Danger(FloatOrd<f32>);

impl Danger {
    pub(crate) fn new(envir: &Envir, time: &Milliseconds, pos: Pos, froms: &[Pos]) -> Self {
        Self(FloatOrd(
            time.0 as f32
                * froms
                    .iter()
                    .map(|from| 1.0 / (envir.walking_cost(pos, *from).f32()))
                    .sum::<f32>(),
        ))
    }

    pub(crate) fn average(&self, time: &Milliseconds) -> Self {
        Self(FloatOrd(self.0 .0 / (time.0 as f32)))
    }
}

impl Add<Self> for Danger {
    type Output = Danger;

    fn add(self, other: Self) -> Danger {
        Danger(FloatOrd(self.0 .0 + other.0 .0))
    }
}

impl Zero for Danger {
    fn zero() -> Self {
        Danger(FloatOrd(0.0))
    }

    fn is_zero(&self) -> bool {
        self.0 == FloatOrd(0.0)
    }
}

#[derive(Component, Debug)]
pub(crate) struct LastEnemy(Pos);
