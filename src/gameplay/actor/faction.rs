use crate::prelude::*;
use bevy::prelude::{Color, Component};
use float_ord::FloatOrd;
use pathfinding::{
    num_traits::Zero,
    prelude::{build_path, dijkstra_all},
};
use std::{mem::discriminant, ops::Add};

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum Intelligence {
    Dumb,
    Smart,
}

pub(crate) struct Strategy {
    pub(crate) intent: Intent,
    pub(crate) action: PlannedAction,
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
    /// in order of consideration
    const ALL: [Self; 4] = [Self::Attack, Self::Flee, Self::Wander, Self::Wait];
}

#[derive(Clone, Component, Debug, PartialEq)]
pub(crate) enum Faction {
    Human,
    Zombie,
    Animal,
}

impl Faction {
    // like PartialEq and Eq, but private
    fn equals(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }

    pub(crate) fn is_aggressive(&self, health: &Health) -> bool {
        match self {
            Self::Human => health.0.percent() < 0.5,
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

    pub(crate) const fn color(&self) -> Color {
        match self {
            Self::Human => DEFAULT_TEXT_COLOR,
            Self::Zombie => FILTHY_COLOR,
            Self::Animal => WARN_TEXT_COLOR,
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
        actor: &ActorItem,
    ) -> Option<(PlannedAction, LastEnemy)> {
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
                    .path(
                        *actor.pos,
                        enemy_pos,
                        self.intelligence(),
                        |_| true,
                        actor.speed(),
                    )
                    .map(|path| (memory, path))
            })
            .min_by_key(|(memory, path)| (*memory, path.duration.0))
            .and_then(|(_, path)| {
                let last_enemy = LastEnemy(path.destination);
                let nbor = envir.to_nbor(*actor.pos, path.first).expect("Nbors");
                if path.first == path.destination {
                    if factions
                        .iter()
                        .filter(|(_, faction)| faction != &self)
                        .any(|(pos, _)| pos == &path.destination)
                    {
                        Some((PlannedAction::Attack { target: nbor }, last_enemy))
                    } else {
                        None
                    }
                } else if envir.find_obstacle(path.first).is_some() {
                    Some((
                        if factions.iter().any(|(pos, _)| *pos == path.first) {
                            PlannedAction::Stay {
                                duration: StayDuration::Short,
                            }
                        } else {
                            PlannedAction::Smash { target: nbor }
                        },
                        last_enemy,
                    ))
                } else {
                    Some((PlannedAction::Step { to: nbor }, last_enemy))
                }
            })
    }

    pub(crate) fn flee(
        &self,
        envir: &Envir,
        enemies: &[Pos],
        actor: &ActorItem,
    ) -> Option<PlannedAction> {
        if enemies.is_empty() {
            return None;
        }

        let up_time =
            WalkingCost::new(NborDistance::Up, MoveCost::default()).duration(actor.speed());

        // Higher gives better results but is slower
        let planning_limit: u64 = 5;
        let min_time = Milliseconds((planning_limit - 1) * up_time.0); // included
        let max_time = Milliseconds(planning_limit * up_time.0); // not included

        let graph = dijkstra_all(&(*actor.pos, Milliseconds(0)), |(pos, prev_total_ms)| {
            envir
                .nbors_for_moving(*pos, None, self.intelligence(), actor.speed())
                .filter_map(|(_, nbor_pos, ms)| {
                    let total_ms = *prev_total_ms + ms;
                    if max_time < total_ms {
                        None
                    } else {
                        Some((
                            (nbor_pos, total_ms),
                            Danger::new(envir, &ms, nbor_pos, enemies),
                        ))
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
        let to = build_path(safest_longtime_pos, &graph)
            .get(1)
            .expect("First step (after current position)")
            .0;
        let nbor = envir.to_nbor(*actor.pos, to).expect("Nbors");
        Some(if nbor == Nbor::HERE {
            PlannedAction::Stay {
                duration: StayDuration::Short,
            }
        } else {
            PlannedAction::Step { to: nbor }
        })
    }

    pub(crate) fn wander(
        &self,
        envir: &Envir,
        factions: &[(Pos, &Self)],
        actor: &ActorItem,
    ) -> Option<PlannedAction> {
        if fastrand::u8(0..10) < 3 {
            let wander_options = envir
                .nbors_for_moving(*actor.pos, None, self.intelligence(), actor.speed())
                .filter(|(_, pos, _)| factions.iter().all(|(other_pos, _)| pos != other_pos))
                .map(|(_, pos, _)| pos)
                .collect::<Vec<Pos>>();

            fastrand::choice(wander_options).map(|pos| {
                let nbor = envir.to_nbor(*actor.pos, pos).expect("Nbors");
                if envir.find_character(pos).is_some() {
                    PlannedAction::Attack { target: nbor }
                } else if envir.find_smashable(pos).is_some() {
                    PlannedAction::Smash { target: nbor }
                } else {
                    PlannedAction::Step { to: nbor }
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
        actor: &ActorItem,
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
            Intent::Wait => Some(PlannedAction::Stay {
                duration: StayDuration::Short,
            })
            .map(|action| (action, None)),
        }
        .filter(|(action, _)| match action {
            // prevent fish from acting on land
            PlannedAction::Step { to: nbor }
            | PlannedAction::Attack { target: nbor }
            | PlannedAction::Smash { target: nbor } => {
                let pos = envir.get_nbor(*actor.pos, *nbor).expect("Valid pos");
                actor.aquatic.is_none() || envir.is_water(pos)
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
        enemies: &[Pos],
        actor: &ActorItem,
    ) -> Strategy {
        // println!("{self:?} can see {:?} enemies", enemies.len());
        Intent::ALL
            .into_iter()
            .filter(|intent| self.consider(*intent, actor.health))
            .find_map(|intent| self.attempt(intent, envir, factions, enemies, actor))
            .expect("Fallback intent")
    }

    /// The name of the first enemy, if any
    pub(crate) fn enemy_name(
        &self,
        currently_visible_builder: &CurrentlyVisibleBuilder,
        factions: &[(Pos, &Self)],
        actor: &ActorItem,
    ) -> Option<Fragment> {
        let currently_visible = currently_visible_builder.for_npc(*actor.pos);
        let player_pos = currently_visible_builder.player_pos();

        factions
            .iter()
            .filter(|(_, other_faction)| self.dislikes(other_faction))
            .map(|(enemy_pos, _)| enemy_pos)
            .copied()
            .filter(|enemy_pos| {
                actor.aquatic.is_none() || currently_visible_builder.envir.is_water(*enemy_pos)
            })
            .filter(|enemy_pos| currently_visible.can_see(*enemy_pos, None) == Visible::Seen)
            .min_by_key(|enemy_pos| player_pos.vision_distance(*enemy_pos))
            .and_then(|enemy_pos| {
                currently_visible_builder
                    .envir
                    .find_character(enemy_pos)
                    .map(|(_, name)| name.single(enemy_pos))
            })
    }

    pub(crate) fn enemies(
        &self,
        currently_visible_builder: &CurrentlyVisibleBuilder,
        factions: &[(Pos, &Self)],
        actor: &ActorItem,
    ) -> Vec<Pos> {
        let currently_visible = currently_visible_builder.for_npc(*actor.pos);

        factions
            .iter()
            .filter(|(_, other_faction)| self.dislikes(other_faction))
            .map(|(enemy_pos, _)| enemy_pos)
            .copied()
            .filter(|enemy_pos| {
                actor.aquatic.is_none() || currently_visible_builder.envir.is_water(*enemy_pos)
            })
            .filter(|enemy_pos| currently_visible.can_see(*enemy_pos, None) == Visible::Seen)
            .collect::<Vec<Pos>>()
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
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(FloatOrd(self.0 .0 + other.0 .0))
    }
}

impl Zero for Danger {
    fn zero() -> Self {
        Self(FloatOrd(0.0))
    }

    fn is_zero(&self) -> bool {
        self.0 == FloatOrd(0.0)
    }
}

#[derive(Component, Debug)]
pub(crate) struct LastEnemy(Pos);
