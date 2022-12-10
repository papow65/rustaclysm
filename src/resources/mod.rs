mod debug;
mod envir;
mod explored;
mod item_infos;
mod location;
mod spawner;
mod zone_level_names;

use crate::prelude::*;
use bevy::{
    ecs::system::{Resource, SystemParam},
    prelude::*,
    utils::HashMap,
};
use std::iter::once;

pub(crate) use self::{
    debug::*, envir::*, explored::*, item_infos::*, location::*, spawner::*, zone_level_names::*,
};

pub(crate) enum Collision {
    Pass,
    //Fall(Pos), // todo
    Blocked(Label),
    Ledged,
}

// pickup
#[derive(SystemParam)]
pub(crate) struct Hierarchy<'w, 's> {
    pub(crate) picked: Query<'w, 's, (Entity, &'static Label, &'static Containable)>,
    pub(crate) children: Query<'w, 's, (&'static Parent, &'static Containable)>,
}

#[derive(Debug, Default, Resource)]
pub(crate) struct InstructionQueue {
    queue: Vec<QueuedInstruction>,
    continuous: Vec<QueuedInstruction>,
}

impl InstructionQueue {
    pub(crate) fn add(&mut self, instruction: QueuedInstruction) {
        // Wait for an instruction to be processed until adding a duplicate when holding a key down.
        if !self.continuous.contains(&instruction) || !self.queue.contains(&instruction) {
            self.queue.insert(0, instruction);
            self.continuous.push(instruction);
        }
    }

    pub(crate) fn interrupt(&mut self, instruction: QueuedInstruction) {
        self.continuous.retain(|k| *k != instruction);
    }

    pub(crate) fn pop(&mut self) -> Option<QueuedInstruction> {
        self.queue.pop()
    }

    pub(crate) fn log_if_long(&self) {
        if 1 < self.queue.len() {
            println!("Unprocessed key codes: {:?}", self.queue);
        }
    }
}

#[derive(Resource)]
pub(crate) struct Timeouts {
    start: Milliseconds,
    m: HashMap<Entity, Milliseconds>,
}

impl Timeouts {
    pub(crate) fn new(turn: u64) -> Self {
        Self {
            start: Milliseconds(1000 * turn),
            m: HashMap::default(),
        }
    }

    pub(crate) fn add(&mut self, entity: Entity, timeout: Milliseconds) {
        self.m.get_mut(&entity).unwrap().0 += timeout.0;
    }

    /// Does not 'pop' the entity
    /// Adds a timeout for untracked entities
    pub(crate) fn next(&mut self, entities: &[Entity]) -> Option<Entity> {
        self.m.retain(|e, _| entities.contains(e));
        let time = self.time();
        entities
            .iter()
            .copied()
            .min_by_key(|e| *self.m.entry(*e).or_insert(time))
    }

    pub(crate) fn time(&self) -> Milliseconds {
        self.m.values().min().copied().unwrap_or(self.start)
    }
}

#[derive(SystemParam)]
pub(crate) struct Characters<'w, 's> {
    pub(crate) c: Query<
        'w,
        's,
        (
            Entity,
            &'static Label,
            &'static Pos,
            &'static Speed,
            &'static Health,
            &'static Faction,
            &'static Container,
        ),
    >,
}

impl<'w, 's> Characters<'w, 's> {
    pub(crate) fn collect_factions(&'s self) -> Vec<(Pos, &'s Faction)> {
        self.c
            .iter()
            .map(|(_, _, p, _, _, f, _)| (*p, f))
            .collect::<Vec<(Pos, &'s Faction)>>()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RelativeRay {
    path: Vec<Pos>,
}

impl RelativeRay {
    fn to_segment(&self, others: &HashMap<Pos, Self>) -> RelativeSegment {
        let mut preceding = None;
        for potential_preceding in self.path.iter().rev().skip(1) {
            let potential_sub_path = others.get(potential_preceding).unwrap();
            if self.path.starts_with(&potential_sub_path.path) {
                preceding = Some(*potential_preceding);
                break; // stop at the first (longest) match
            }
        }

        let skipped = if let Some(prededing) = preceding {
            others.get(&prededing).unwrap().path.len() - 1
        } else {
            0
        };

        let mut segment = if skipped == 0 {
            self.path.clone()
        } else {
            self.path[skipped..].to_vec()
        };
        // The last pos doen't need to be transparent.
        segment.pop();

        let down_pairs = once(Pos::ORIGIN)
            .chain(self.path.iter().copied())
            .zip(self.path.iter().copied())
            .skip(skipped)
            .filter(|(current, next)| current.level != next.level)
            .map(|(current, next)| {
                let level = current.level.max(next.level);
                (
                    Pos::new(current.x, level, current.z),
                    Pos::new(next.x, level, next.z),
                )
            })
            .collect::<Vec<_>>();

        //println!("{:?} {:?} {}", &self.path, &preceding, skipped);

        RelativeSegment {
            preceding,
            segment,
            down_pairs,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RelativeSegment {
    preceding: Option<Pos>,
    segment: Vec<Pos>,
    down_pairs: Vec<(Pos, Pos)>,
}

#[derive(Resource)]
pub(crate) struct RelativeSegments {
    segments: HashMap<Pos, RelativeSegment>,
}

impl RelativeSegments {
    pub(crate) fn new() -> Self {
        let mut rays = HashMap::default();
        let origin = Pos::new(0, Level::ZERO, 0);
        for x in -MAX_VISIBLE_DISTANCE..=MAX_VISIBLE_DISTANCE {
            for y in Level::ALL {
                for z in -MAX_VISIBLE_DISTANCE..=MAX_VISIBLE_DISTANCE {
                    let to = Pos::new(x, y, z);
                    if !origin.in_visible_range(to) {
                        continue;
                    }

                    rays.insert(
                        to,
                        RelativeRay {
                            path: if to == origin {
                                vec![]
                            } else {
                                origin.straight(to).collect::<Vec<Pos>>()
                            },
                        },
                    );
                }
            }
        }

        assert!(
            rays.get(&Pos::ORIGIN) == Some(&RelativeRay { path: Vec::new() }),
            "{:?}",
            rays.get(&Pos::ORIGIN)
        );

        assert!(
            rays.get(&Pos::new(1, Level::ZERO, 0))
                == Some(&RelativeRay {
                    path: vec![Pos::new(1, Level::ZERO, 0)],
                }),
            "{:?}",
            rays.get(&Pos::new(1, Level::ZERO, 0))
        );

        assert!(
            rays.get(&Pos::new(2, Level::ZERO, 0))
                == Some(&RelativeRay {
                    path: vec![Pos::new(1, Level::ZERO, 0), Pos::new(2, Level::ZERO, 0)],
                }),
            "{:?}",
            rays.get(&Pos::new(2, Level::ZERO, 0))
        );

        let lower_bound = (2 * MAX_VISIBLE_DISTANCE as usize + 1).pow(2) * Level::AMOUNT / 2;
        let upper_bound = (2 * MAX_VISIBLE_DISTANCE as usize + 1).pow(2) * Level::AMOUNT;
        assert!(lower_bound < rays.len(), "{} {}", lower_bound, rays.len());
        assert!(rays.len() < upper_bound, "{} {}", rays.len(), upper_bound);
        for nbor in Nbor::ALL {
            let nbor = Pos::ORIGIN.raw_nbor(&nbor).unwrap();
            assert!(rays.contains_key(&nbor), "{nbor:?}");
        }

        let mut segments = HashMap::<Pos, RelativeSegment>::default();
        for (pos, relativeray) in rays.iter() {
            segments.insert(*pos, relativeray.to_segment(&rays));
        }

        for i in 2..=60 {
            let pos = Pos::new(i, Level::ZERO, 0);
            assert!(
                segments.get(&pos)
                    == Some(&RelativeSegment {
                        preceding: Some(Pos::new(i - 1, Level::ZERO, 0)),
                        segment: vec![Pos::new(i - 1, Level::ZERO, 0)],
                        down_pairs: Vec::new()
                    }),
                "{:?} -> {:?}",
                pos,
                segments.get(&pos)
            );
        }

        for level in 2..=10 {
            let level = Level::new(level);
            let pos = Pos::new(0, level, 0);
            assert!(
                matches!(
                    segments.get(&pos),
                    Some(&RelativeSegment {
                        ref preceding,
                        ref segment,
                        ..
                    }) if preceding == &Some(Pos::new(0, Level::new(level.h - 1), 0)) && segment == &vec![Pos::new(0, Level::new(level.h - 1), 0)]
                ),
                "{:?} -> {:?}",
                pos,
                segments.get(&pos)
            );
        }

        Self { segments }
    }
}
