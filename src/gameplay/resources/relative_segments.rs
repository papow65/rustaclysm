use crate::prelude::*;
use bevy::{ecs::system::Resource, utils::HashMap};
use std::iter::once;

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
        // The last pos doen't need to be transparent.
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
    pub(crate) segments: HashMap<PosOffset, RelativeSegment>,
}

impl RelativeSegments {
    pub(crate) fn new() -> Self {
        let mut rays = HashMap::default();
        for x in -MAX_VISIBLE_DISTANCE..=MAX_VISIBLE_DISTANCE {
            for y in Level::ALL {
                for z in -MAX_VISIBLE_DISTANCE..=MAX_VISIBLE_DISTANCE {
                    let to = Pos::new(x, y, z);
                    if !Pos::ORIGIN.in_visible_range(to) {
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
            Some(&RelativeRay { path: Vec::new() })
        );

        assert_eq!(
            rays.get(&Pos::new(1, Level::ZERO, 0)),
            Some(&RelativeRay {
                path: vec![Pos::new(1, Level::ZERO, 0)],
            })
        );

        assert_eq!(
            rays.get(&Pos::new(2, Level::ZERO, 0)),
            Some(&RelativeRay {
                path: vec![Pos::new(1, Level::ZERO, 0), Pos::new(2, Level::ZERO, 0)],
            })
        );

        let lower_bound = (2 * MAX_VISIBLE_DISTANCE as usize + 1).pow(2) * Level::AMOUNT / 2;
        let upper_bound = (2 * MAX_VISIBLE_DISTANCE as usize + 1).pow(2) * Level::AMOUNT;
        assert!(lower_bound < rays.len(), "{lower_bound} {}", rays.len());
        assert!(rays.len() < upper_bound, "{} {upper_bound}", rays.len());
        for nbor in Nbor::ALL {
            let nbor = Pos::ORIGIN.raw_nbor(&nbor).unwrap();
            assert!(rays.contains_key(&nbor), "{nbor:?}");
        }

        let mut segments = HashMap::<PosOffset, RelativeSegment>::default();
        for (pos, relativeray) in rays.iter() {
            segments.insert(*pos - Pos::ORIGIN, relativeray.to_segment(&rays));
        }

        for i in 2..=60 {
            let offset = Pos::new(i, Level::ZERO, 0) - Pos::ORIGIN;
            assert_eq!(
                segments.get(&offset),
                Some(&RelativeSegment {
                    preceding: Some(PosOffset {
                        x: i - 1,
                        level: LevelOffset::ZERO,
                        z: 0
                    }),
                    segment: vec![PosOffset {
                        x: i - 1,
                        level: LevelOffset::ZERO,
                        z: 0
                    }],
                    down_pairs: Vec::new()
                }),
            );
        }

        for level in 2..=10 {
            let level = Level::new(level);
            let offset = Pos::new(0, level, 0) - Pos::ORIGIN;
            assert!(
                matches!(
                    segments.get(&offset),
                    Some(RelativeSegment {
                        preceding,
                        segment,
                        ..
                    }) if preceding == &Some(PosOffset{x: 0, level: LevelOffset{h: level.h - 1}, z: 0}) && segment == &vec![PosOffset{x: 0, level: LevelOffset{h: level.h - 1}, z: 0}]
                ),
                "{:?} -> {:?}",
                offset,
                segments.get(&offset)
            );
        }

        Self { segments }
    }
}
