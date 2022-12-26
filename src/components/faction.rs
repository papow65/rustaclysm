use crate::prelude::*;
use bevy::prelude::Component;
use float_ord::FloatOrd;
use pathfinding::{
    num_traits::Zero,
    prelude::{build_path, dijkstra_all},
};
use rand::prelude::*;
use rand::seq::SliceRandom;
use std::{iter::once, ops::Add};

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
        speed: BaseSpeed,
        factions: &[(Pos, &Self)],
        enemies: &[Pos],
        last_enemy: Option<&LastEnemy>,
    ) -> Option<(Action, LastEnemy)> {
        enemies
            .iter()
            .copied()
            .chain(last_enemy.iter().map(|l| l.0))
            .filter_map(|enemy_pos| envir.path(start_pos, enemy_pos, self.intelligence(), speed))
            .min_by_key(|path| path.duration.0)
            .and_then(|path| {
                let last_enemy = LastEnemy(path.destination);

                //println!("{:?}->{:?}", path.first, path.destination,);
                if path.first == path.destination {
                    if factions.iter().any(|(pos, _)| pos == &path.destination) {
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

    pub(crate) fn flee(
        &self,
        envir: &Envir,
        start_pos: Pos,
        speed: BaseSpeed,
        enemies: &[Pos],
    ) -> Option<Action> {
        if enemies.is_empty() {
            return None;
        }

        let up_time = WalkingCost::new(&NborDistance::Up, MoveCost::default()).duration(speed);

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
        Some(if target == start_pos {
            Action::Stay
        } else {
            Action::Step { target }
        })
    }

    pub(crate) fn wander(&self, envir: &Envir, start_pos: Pos, speed: BaseSpeed) -> Option<Action> {
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
        speed: BaseSpeed,
        aquatic: Option<&Aquatic>,
        factions: &[(Pos, &Self)],
        enemies: &[Pos],
        last_enemy: Option<&LastEnemy>,
    ) -> Option<Strategy> {
        match intent {
            Intent::Attack => self
                .attack(envir, start_pos, speed, factions, enemies, last_enemy)
                .map(|(action, last_enemy)| (action, Some(last_enemy))),
            Intent::Flee => self
                .flee(envir, start_pos, speed, enemies)
                .map(|action| (action, None)),
            Intent::Wander => self
                .wander(envir, start_pos, speed)
                .map(|action| (action, None)),
            Intent::Wait => Some(Action::Stay).map(|action| (action, None)),
        }
        .map(|(action, last_enemy)| Strategy {
            intent,
            action,
            last_enemy,
        })
        .filter(|strategy| {
            if let Action::Step { target } = strategy.action {
                // prevent fish from walking on land
                aquatic.is_none() || envir.is_water(target)
            } else {
                true
            }
        })
    }

    pub(crate) fn behave<'f>(
        &self,
        envir: &Envir,
        start_pos: Pos,
        speed: BaseSpeed,
        health: &Health,
        aquatic: Option<&Aquatic>,
        factions: &[(Pos, &'f Self)],
        last_enemy: Option<&LastEnemy>,
    ) -> Strategy {
        let currently_visible = envir.currently_visible(start_pos);

        let enemies = factions
            .iter()
            .filter(|(_, other_faction)| self.dislikes(other_faction))
            .map(|(enemy_pos, _)| enemy_pos)
            .copied()
            .filter(|enemy_pos| aquatic.is_none() || envir.is_water(*enemy_pos))
            .filter(|enemy_pos| currently_visible.can_see(*enemy_pos) == Visible::Seen)
            .collect::<Vec<Pos>>();
        //println!("{self:?} can see {:?} enemies", enemies.len());

        self.intents(health)
            .find_map(|intent| {
                self.attempt(
                    intent, envir, start_pos, speed, aquatic, factions, &enemies, last_enemy,
                )
            })
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
