use crate::prelude::*;
use bevy::{ecs::system::Resource, prelude::*, utils::HashMap};
use std::{array, iter::once, time::Instant};

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
                preceding = Some(*potential_preceding - Pos::ORIGIN);
                break; // stop at the first (longest) match
            }
        }

        let skipped = if let Some(preceding) = preceding {
            others
                .get(&Pos::ORIGIN.offset(preceding).unwrap())
                .unwrap()
                .path
                .len()
                - 1
        } else {
            0
        };

        let mut segment = if skipped == 0 {
            self.path.clone()
        } else {
            self.path[skipped..].to_vec()
        };
        // The last pos is not required to be transparent.
        segment.pop();
        let segment = segment
            .iter()
            .map(|pos| *pos - Pos::ORIGIN)
            .collect::<Vec<_>>();

        let down_pairs = once(Pos::ORIGIN)
            .chain(self.path.iter().copied())
            .zip(self.path.iter().copied())
            .skip(skipped)
            .filter(|(current, next)| current.level != next.level)
            .map(|(current, next)| {
                let level = current.level.max(next.level);
                (
                    Pos::new(current.x, level, current.z) - Pos::ORIGIN,
                    Pos::new(next.x, level, next.z) - Pos::ORIGIN,
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
    pub(crate) preceding: Option<PosOffset>,
    pub(crate) segment: Vec<PosOffset>,
    pub(crate) down_pairs: Vec<(PosOffset, PosOffset)>,
}

#[derive(Resource)]
pub(crate) struct RelativeSegments {
    pub(crate) segments: [HashMap<PosOffset, RelativeSegment>; Self::SIZE],
}

impl RelativeSegments {
    const SIZE: usize = MAX_VISIBLE_DISTANCE as usize + 1;

    fn new() -> Self {
        let start = Instant::now();

        let rays = Self::rays();

        let mut segments = array::from_fn(|_| HashMap::<PosOffset, RelativeSegment>::default());
        for (pos, relativeray) in &rays {
            let segment = relativeray.to_segment(&rays);
            let distance = Pos::ORIGIN.vision_distance(*pos);
            for index in distance..=(MAX_VISIBLE_DISTANCE as usize) {
                segments
                    .get_mut(index)
                    .unwrap()
                    .insert(*pos - Pos::ORIGIN, segment.clone());
            }
        }

        let duration = start.elapsed();
        println!("The creation of RelativeSegments took {duration:?}");

        Self { segments }
    }

    fn rays() -> HashMap<Pos, RelativeRay> {
        let mut rays = HashMap::default();
        for x in -MAX_VISIBLE_DISTANCE..=MAX_VISIBLE_DISTANCE {
            for y in Level::ALL {
                for z in -MAX_VISIBLE_DISTANCE..=MAX_VISIBLE_DISTANCE {
                    let to = Pos::new(x, y, z);
                    if (MAX_VISIBLE_DISTANCE as usize) < Pos::ORIGIN.vision_distance(to) {
                        continue;
                    }

                    rays.insert(
                        to,
                        RelativeRay {
                            path: if to == Pos::ORIGIN {
                                vec![]
                            } else {
                                Pos::ORIGIN.straight(to).collect::<Vec<Pos>>()
                            },
                        },
                    );
                }
            }
        }

        assert_eq!(
            rays.get(&Pos::ORIGIN),
            Some(&RelativeRay { path: Vec::new() }),
            "The ray at the origin should be empty"
        );

        let relative_ray = RelativeRay {
            path: vec![Pos::new(1, Level::ZERO, 0)],
        };
        assert_eq!(
            rays.get(&Pos::new(1, Level::ZERO, 0)),
            Some(&relative_ray),
            "The ray at (1, 0, 0) should consist of only (1, 0, 0) itself"
        );

        let relative_ray = RelativeRay {
            path: vec![Pos::new(1, Level::ZERO, 0), Pos::new(2, Level::ZERO, 0)],
        };
        assert_eq!(
            rays.get(&Pos::new(2, Level::ZERO, 0)),
            Some(&relative_ray),
            "The ray at (2, 0, 0) should consist of only (1, 0, 0) and (2, 0, 0)"
        );

        let upper_bound = (2 * MAX_VISIBLE_DISTANCE as usize + 1).pow(2) * Level::AMOUNT;
        assert!(
            rays.len() <= upper_bound,
            "{MAX_VISIBLE_DISTANCE} {} {upper_bound}",
            rays.len()
        );

        for nbor in Nbor::ALL {
            let nbor = Pos::ORIGIN.raw_nbor(nbor).unwrap();
            assert!(rays.contains_key(&nbor), "{MAX_VISIBLE_DISTANCE} {nbor:?}");
        }

        rays
    }
}

impl FromWorld for RelativeSegments {
    fn from_world(_world: &mut World) -> Self {
        Self::new()
    }
}
