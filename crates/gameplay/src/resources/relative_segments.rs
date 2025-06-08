use bevy::{platform::collections::HashMap, prelude::Resource};
use gameplay_location::{Level, LevelOffset, Nbor, Pos, PosOffset, VisionDistance};
use std::{array, iter::once};
use util::AsyncNew;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RelativeRay {
    path: Vec<PosOffset>,
}

impl RelativeRay {
    fn to_segment(&self, others: &HashMap<PosOffset, Self>) -> RelativeSegment {
        let mut preceding = None;
        for &potential_preceding in self.path.iter().rev().skip(1) {
            let potential_sub_path = others
                .get(&potential_preceding)
                .expect("Shorter paths should already have been processed");
            if self.path.starts_with(&potential_sub_path.path) {
                preceding = Some(potential_preceding);
                break; // stop at the first (longest) match
            }
        }

        let skipped = if let Some(preceding) = preceding {
            others
                .get(&preceding)
                .expect("Shorter paths should already have been processed")
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

        let down_pairs = once(PosOffset::HERE)
            .chain(self.path.iter().copied())
            .zip(self.path.iter().copied())
            .skip(skipped)
            .filter(|(current, next)| current.level != next.level)
            .map(|(mut current, mut next)| {
                let max_level = current.level.max(next.level);
                current.level = max_level;
                next.level = max_level;
                (current, next)
            })
            .collect::<Vec<_>>();

        //trace!("{:?} {:?} {}", &self.path, &preceding, skipped);

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

#[derive(Debug, Resource)]
pub(crate) struct RelativeSegments {
    pub(crate) segments: [HashMap<PosOffset, RelativeSegment>; Self::SIZE],
}

impl RelativeSegments {
    const SIZE: usize = VisionDistance::MAX_VISION_TILES as usize + 1; // for 0..=MAX_VISION_MAX

    /// This might take a couple of seconds
    fn new() -> Self {
        let rays = Self::rays();

        let mut segments = array::from_fn(|_| HashMap::<PosOffset, RelativeSegment>::default());
        for (&pos_offset, relativeray) in &rays {
            let segment = relativeray.to_segment(&rays);
            let distance = VisionDistance::from_offset(pos_offset).as_tiles();
            for index in distance..=(VisionDistance::MAX_VISION_TILES as usize) {
                segments
                    .get_mut(index)
                    .expect("'index' should be a valid index")
                    .insert(pos_offset, segment.clone());
            }
        }

        Self { segments }
    }

    fn rays() -> HashMap<PosOffset, RelativeRay> {
        let mut rays = HashMap::default();
        for x in VisionDistance::MAX_VISION_RANGE {
            for level in Level::ALL {
                for z in VisionDistance::MAX_VISION_RANGE {
                    let to = PosOffset {
                        x,
                        level: level - Level::ZERO,
                        z,
                    };
                    if VisionDistance::from_offset(to)
                        .in_range(VisionDistance::MAX_VISION_TILES as usize)
                    {
                        rays.insert(
                            to,
                            RelativeRay {
                                path: if to == PosOffset::HERE {
                                    vec![]
                                } else {
                                    Pos::ORIGIN
                                        .straight(
                                            Pos::ORIGIN.offset(to).expect(
                                                "Pos from origin and offset should be valid",
                                            ),
                                        )
                                        .map(|pos| pos - Pos::ORIGIN)
                                        .collect::<Vec<_>>()
                                },
                            },
                        );
                    }
                }
            }
        }

        assert_eq!(
            rays.get(&PosOffset::HERE),
            Some(&RelativeRay { path: Vec::new() }),
            "The ray at the origin should be empty"
        );

        let relative_ray = RelativeRay {
            path: vec![PosOffset {
                x: 1,
                level: LevelOffset::ZERO,
                z: 0,
            }],
        };
        assert_eq!(
            rays.get(&PosOffset {
                x: 1,
                level: LevelOffset::ZERO,
                z: 0
            }),
            Some(&relative_ray),
            "The ray at (1, 0, 0) should consist of only (1, 0, 0) itself"
        );

        let relative_ray = RelativeRay {
            path: vec![
                PosOffset {
                    x: 1,
                    level: LevelOffset::ZERO,
                    z: 0,
                },
                PosOffset {
                    x: 2,
                    level: LevelOffset::ZERO,
                    z: 0,
                },
            ],
        };
        assert_eq!(
            rays.get(&PosOffset {
                x: 2,
                level: LevelOffset::ZERO,
                z: 0
            }),
            Some(&relative_ray),
            "The ray at (2, 0, 0) should consist of only (1, 0, 0) and (2, 0, 0)"
        );

        let upper_bound =
            (2 * VisionDistance::MAX_VISION_TILES as usize + 1).pow(2) * Level::AMOUNT;
        assert!(
            rays.len() <= upper_bound,
            "{} {} {upper_bound}",
            VisionDistance::MAX_VISION_TILES,
            rays.len()
        );

        for nbor in Nbor::ALL {
            let nbor = Pos::ORIGIN
                .raw_nbor(nbor)
                .expect("All nbors from te origin should be valid");
            assert!(
                rays.contains_key(&(nbor - Pos::ORIGIN)),
                "{} {nbor:?}",
                VisionDistance::MAX_VISION_TILES,
            );
        }

        rays
    }
}

impl AsyncNew<Self> for RelativeSegments {
    async fn async_new() -> Self {
        async move { Self::new() }.await
    }
}
