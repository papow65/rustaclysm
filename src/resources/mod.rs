mod debug;
mod envir;
mod location;
mod spawner;

use crate::prelude::*;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::utils::HashMap;

pub use self::{debug::*, envir::*, location::*, spawner::*};

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
        let origin = Pos::new(0, Level::ZERO, 0);
        for x in -60..=60 {
            for y in Level::ALL {
                for z in -60..=60 {
                    let to = Pos::new(x, y, z);

                    if 14_400
                        < 4 * to.x.pow(2) + 25 * i32::from(to.level.h).pow(2) + 4 * to.z.pow(2)
                    {
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

    pub fn ray(
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
