mod debug;
mod envir;
mod location;
mod spawner;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::utils::HashMap;

use super::components::{Containable, Container, Faction, Health, Instruction, Label, Pos};
use super::unit::{Distance, Milliseconds, Speed};

pub use debug::*;
pub use envir::*;
pub use location::*;
pub use spawner::*;

pub enum Collision {
    Pass,
    //Fall(Pos), // todo
    Blocked(Label),
    Ledged,
    NoStairsUp,
    NoStairsDown,
}

// pickup
#[derive(SystemParam)]
pub struct Hierarchy<'w, 's> {
    pub picked: Query<'w, 's, (Entity, &'static Label, &'static Containable)>,
    pub children: Query<'w, 's, (&'static Parent, &'static Containable)>,
}

#[derive(Debug)]
pub struct Instructions {
    pub queue: Vec<Instruction>,
}

impl Instructions {
    pub const fn new() -> Self {
        Self { queue: Vec::new() }
    }
}

pub struct Timeouts {
    m: HashMap<Entity, Milliseconds>,
}

impl Timeouts {
    pub fn new() -> Self {
        Self {
            m: HashMap::default(),
        }
    }

    pub fn add(&mut self, entity: Entity, timeout: Milliseconds) {
        self.m.get_mut(&entity).unwrap().0 += timeout.0;
    }

    /// Does not 'pop' the entity
    /// Adds a timeout for untracked entities
    pub fn next(&mut self, entities: &[Entity]) -> Option<Entity> {
        self.m.retain(|e, _| entities.contains(e));
        let time = self.time();
        entities
            .iter()
            .copied()
            .min_by_key(|e| *self.m.entry(*e).or_insert(time))
    }

    pub fn time(&self) -> Milliseconds {
        self.m.values().min().copied().unwrap_or(Milliseconds(0))
    }
}

#[derive(SystemParam)]
pub struct Characters<'w, 's> {
    pub c: Query<
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
    pub fn collect_factions(&'s self) -> Vec<(Pos, &'s Faction)> {
        self.c
            .iter()
            .map(|(_, _, p, _, _, f, _)| (*p, f))
            .collect::<Vec<(Pos, &'s Faction)>>()
    }
}

pub struct RelativeRays(HashMap<Pos, (Vec<Pos>, Vec<(Pos, Pos)>)>);

impl RelativeRays {
    pub fn new() -> Self {
        let mut map: HashMap<Pos, (Vec<Pos>, Vec<(Pos, Pos)>)> = HashMap::default();
        let origin = Pos(0, 0, 0);
        for x in -60..=60 {
            for y in Pos::vertical_range() {
                for z in -60..=60 {
                    let to = Pos(x, y, z);

                    if 14_400 < 4 * to.0.pow(2) + 25 * to.1.pow(2) + 4 * to.2.pow(2) {
                        // more than 60 meter away
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
                        .filter(|(a, b)| a.1 != b.1)
                        .map(|(a, b)| {
                            let y = a.1.max(b.1);
                            (Pos(a.0, y, a.2), Pos(b.0, y, b.2))
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

    pub fn ray(
        &self,
        from: Pos,
        to: Pos,
    ) -> Option<(
        impl Iterator<Item = Pos> + '_,
        impl Iterator<Item = (Pos, Pos)> + '_,
    )> {
        self.0
            .get(&Pos(to.0 - from.0, to.1 - from.1, to.2 - from.2))
            .map(|(line, down)| {
                (
                    line.iter()
                        .map(move |pos| Pos(pos.0 + from.0, pos.1 + from.1, pos.2 + from.2)),
                    down.iter().map(move |(a, b)| {
                        (
                            Pos(a.0 + from.0, a.1 + from.1, a.2 + from.2),
                            Pos(b.0 + from.0, b.1 + from.1, b.2 + from.2),
                        )
                    }),
                )
            })
    }
}
