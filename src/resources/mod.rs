mod debug;
mod envir;
mod explored;
mod item_infos;
mod location;
mod spawner;
mod zone_level_names;

use crate::prelude::*;
use bevy::ecs::system::{Resource, SystemParam};
use bevy::prelude::*;
use bevy::utils::HashMap;

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

#[derive(Resource)]
pub(crate) struct RelativeRays(HashMap<Pos, (Vec<Pos>, Vec<(Pos, Pos)>)>);

impl RelativeRays {
    pub(crate) fn new() -> Self {
        let apporx_level_height_sq =
            (Millimeter::VERTICAL.f32().powi(2) / Millimeter::ADJACENT.f32().powi(2) + 0.5) as i32;
        let mut map: HashMap<Pos, (Vec<Pos>, Vec<(Pos, Pos)>)> = HashMap::default();
        let origin = Pos::new(0, Level::ZERO, 0);
        for x in -60..=60 {
            for y in Level::ALL {
                for z in -60..=60 {
                    let to = Pos::new(x, y, z);

                    // if 61² <= x² + h² + z² {
                    if 61_i32.pow(2)
                        <= to.x.pow(2)
                            + apporx_level_height_sq * i32::from(to.level.h).pow(2)
                            + to.z.pow(2)
                    {
                        // 61 meter or more away
                        continue;
                    }

                    let line = if to == origin {
                        Vec::new()
                    } else {
                        origin.straight(to).collect::<Vec<Pos>>()
                    };

                    let down = std::iter::once(origin)
                        .chain(line.iter().copied())
                        .zip(line.iter().copied())
                        .filter(|(a, b)| a.level != b.level)
                        .map(|(a, b)| {
                            let level = a.level.max(b.level);
                            (Pos::new(a.x, level, a.z), Pos::new(b.x, level, b.z))
                        })
                        .collect::<Vec<(Pos, Pos)>>();

                    let between = line
                        .iter()
                        .copied()
                        .filter(|&pos| pos != to)
                        .collect::<Vec<Pos>>();
                    map.insert(to, (between, down));
                }
            }
        }

        Self(map)
    }

    pub(crate) fn ray(
        &self,
        from: Pos,
        to: Pos,
    ) -> Option<(
        impl Iterator<Item = Pos> + '_,
        impl Iterator<Item = (Pos, Pos)> + '_,
    )> {
        self.0
            .get(&Pos::new(
                to.x - from.x,
                Level::new(to.level.h - from.level.h),
                to.z - from.z,
            ))
            .map(|(line, down)| {
                (
                    line.iter().map(move |pos| from.offset(*pos).unwrap()),
                    down.iter()
                        .map(move |(a, b)| (from.offset(*a).unwrap(), from.offset(*b).unwrap())),
                )
            })
    }
}
