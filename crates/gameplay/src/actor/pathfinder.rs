use crate::{Intelligence, QueuedInstruction};
use bevy::prelude::debug;
use cdda_json_files::MoveCost;
use fastrand::f32 as rand_f32;
use gameplay_common::WalkingCost;
use gameplay_location::{HorizontalDirection, Nbor, NborDistance, Pos};
use gameplay_world::Envir;
use pathfinding::prelude::astar;
use units::{Duration, Speed};

pub(crate) struct Pathfinder<'w, 's> {
    envir: &'s Envir<'w, 's>,
}

impl<'w, 's> Pathfinder<'w, 's> {
    pub(crate) const fn new(envir: &'s Envir<'w, 's>) -> Self {
        Self { envir }
    }

    pub(crate) fn nbors_for_moving(
        &'s self,
        pos: Pos,
        destination: Option<Pos>,
        intelligence: Intelligence,
        speed: Speed,
    ) -> impl Iterator<Item = (Nbor, Pos, Duration)> + use<'s> {
        self.envir
            .nbors_if(pos, move |nbor| {
                self.envir.exists(pos) && {
                    let at_destination = Some(nbor) == destination;
                    match intelligence {
                        Intelligence::Smart => {
                            self.envir.is_accessible(nbor)
                                && (at_destination || self.envir.find_obstacle(nbor).is_none())
                        }
                        Intelligence::Dumb => at_destination || !self.envir.is_opaque(nbor),
                    }
                }
            })
            .map(move |(nbor, npos, walking_cost)| (nbor, npos, walking_cost.duration(speed)))
    }

    pub(crate) fn directions_for_item_handling(
        &'s self,
        pos: Pos,
    ) -> impl Iterator<Item = (HorizontalDirection, Pos)> + use<'s> {
        self.envir.nbors(pos).filter_map(|(nbor, npos, _)| {
            HorizontalDirection::try_from(nbor)
                .ok()
                .map(|direction| (direction, npos))
        })
    }

    pub(crate) fn nbors_for_exploring(
        &'s self,
        pos: Pos,
        instruction: &'s QueuedInstruction,
    ) -> impl Iterator<Item = Nbor> + use<'s> {
        self.envir
            .nbors_if(pos, move |nbor| match instruction {
                QueuedInstruction::Attack => {
                    nbor != pos && self.envir.find_character(nbor).is_some()
                }
                QueuedInstruction::Smash => self.envir.find_smashable(nbor).is_some(),
                QueuedInstruction::Pulp => self.envir.find_pulpable(nbor).is_some(),
                QueuedInstruction::Close => self.envir.find_closeable(nbor).is_some(),
                QueuedInstruction::StartCraft { .. } => {
                    self.envir.is_accessible(nbor) && !self.envir.is_water(nbor)
                }
                _ => panic!("unexpected instruction {instruction:?}"),
            })
            .map(move |(nbor, _npos, _distance)| nbor)
    }

    /// `WalkingCost` without regard for obstacles or stairs, unless they are nbors
    pub(crate) fn walking_cost(&self, from: Pos, to: Pos) -> WalkingCost {
        let dx = u64::from(from.x.abs_diff(to.x));
        let dz = u64::from(from.z.abs_diff(to.z));
        let diagonal = dx.min(dz);
        let adjacent = dx.max(dz) - diagonal;

        let dy = (to.level - from.level).h;
        let up = dy.max(0) as u64;
        let down = (-dy).max(0) as u64;

        let move_cost = if diagonal + adjacent + up + down == 1 {
            // nbors, the precise value matters in some cases
            // Dumb creatures may try to use paths that do not have a floor.
            self.envir
                .find_accessibles(to)
                .map_or_else(MoveCost::default, |floor| {
                    floor
                        .move_cost
                        .adjust(self.envir.find_hurdles(to).map(|h| h.0))
                })
        } else {
            // estimate
            MoveCost::default()
        };

        [
            (NborDistance::Up, up),
            (NborDistance::Adjacent, adjacent),
            (NborDistance::Diagonal, diagonal),
            (NborDistance::Down, down),
        ]
        .into_iter()
        .map(|(nd, amount)| WalkingCost::new(nd, move_cost) * amount)
        .sum()
    }

    pub(crate) fn path<F>(
        &self,
        from: Pos,
        to: Pos,
        intelligence: Intelligence,
        seen: F,
        speed: Speed,
        stay_duration: Duration,
    ) -> Option<MovementPath>
    where
        F: Fn(Pos) -> bool,
    {
        if to == from {
            return None;
        }

        let nbors_fn = |pos: &Pos| {
            self.nbors_for_moving(*pos, Some(to), intelligence, speed)
                .filter(|(_, pos, _)| seen(*pos))
                .map(|(_, pos, cost)| (pos, cost))
        };
        let estimated_duration_fn = |&pos: &Pos| {
            self.walking_cost(from, pos)
                .duration(speed)
                .max(stay_duration)
                + self.walking_cost(pos, to).duration(speed)
        };

        //trace!("dumb? {dumb:?} @{from:?}");
        if intelligence == Intelligence::Smart && seen(to) {
            MovementPath::plan(from, nbors_fn, estimated_duration_fn, to)
        } else {
            MovementPath::improvize(nbors_fn(&from), estimated_duration_fn, from, to)
        }
    }
}

#[derive(Debug)]
pub(crate) struct MovementPath {
    pub(crate) first: Pos,
    pub(crate) duration: Duration,
    pub(crate) destination: Pos,
}

impl MovementPath {
    pub(crate) fn plan<FN, IN, FH>(
        from: Pos,
        successors: FN,
        heuristic: FH,
        destination: Pos,
    ) -> Option<Self>
    where
        FN: FnMut(&Pos) -> IN,
        IN: Iterator<Item = (Pos, Duration)>,
        FH: FnMut(&Pos) -> Duration,
    {
        if let Some((mut steps, duration)) =
            astar(&from, successors, heuristic, |&pos| pos == destination)
        {
            assert!(
                2 <= steps.len(),
                "Movement steps should contain both the start and end point"
            );
            assert_eq!(
                steps.remove(0),
                from,
                "The first step should match the starting point"
            );
            Some(Self {
                first: *steps
                    .first()
                    .expect("The last step should always be present"),
                duration,
                destination,
            })
        } else {
            None
        }
    }

    pub(crate) fn improvize<I, FH>(
        nbors: I,
        mut heuristic: FH,
        from: Pos,
        destination: Pos,
    ) -> Option<Self>
    where
        I: Iterator<Item = (Pos, Duration)>,
        FH: FnMut(&Pos) -> Duration,
    {
        let paths = nbors
            .map(|(first, _)| Self {
                first,
                duration: heuristic(&first),
                destination,
            })
            //.inspect(|path| {
            //    trace!(
            //        "MovementPath::improvize {from:?} by {:?} to {destination:?} @ {:?}",
            //        path.first, path.duration
            //    );
            //})
            .collect::<Vec<_>>();

        let shortest = paths
            .iter()
            .min_by_key(|path| (path.first == from, path.duration))?
            .duration;
        let offset = (destination - from).vec3();
        let all_best = paths
            .into_iter()
            .filter(|path| path.duration == shortest)
            .map(|path| {
                if path.first == destination {
                    (1.0, path)
                } else {
                    let match_with_straight_line = offset
                        .angle_between((destination - path.first).vec3())
                        .cos();
                    (match_with_straight_line, path)
                }
            })
            .scan(0.0, |total, (score, path)| {
                *total += score;
                Some((score, path, *total))
            })
            .inspect(|(score, path, total)| {
                debug!(
                    "{from:?}->{:?}->{destination:?}  {score:?} / {total:?}",
                    path.first
                );
            })
            .collect::<Vec<_>>();
        let pick = all_best.last().expect("Not empty").2 * rand_f32();
        all_best
            .into_iter()
            .find(|(_, _, total)| pick <= *total)
            .map(|(_, path, _)| path)
    }
}
