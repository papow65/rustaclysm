use bevy::prelude::{Changed, Entity, Local, Or, Parent, Query, Res, With};
use std::time::{Duration, Instant};

use super::super::components::{Action, Faction, Label, Obstacle, Pos};
use super::super::resources::{Location, StdInstant};

use super::log_if_slow;

#[allow(dead_code)]
#[allow(clippy::needless_pass_by_value)]
pub fn check_obstacle_location(
    location: Res<Location>,
    items: Query<(Entity, &Pos, Option<&Label>), With<Obstacle>>,
) {
    let start = Instant::now();

    for (a, &a_p, a_s) in items.iter() {
        let (b, _, b_s) = location.get_first(a_p, &items).unwrap();
        assert!(
            a == b,
            "Overlap of {} and {} at {:?}",
            a_s.unwrap_or(&Label::new("?")),
            b_s.unwrap_or(&Label::new("?")),
            a_p
        );
    }

    log_if_slow("check_obstacle_location", start);
}

#[allow(dead_code)]
#[allow(clippy::needless_pass_by_value)]
pub fn check_overlap(all_obstacles: Query<(Entity, &Pos, Option<&Label>), With<Obstacle>>) {
    let start = Instant::now();

    for (a, &a_p, a_s) in all_obstacles.iter() {
        for (b, &b_p, b_s) in all_obstacles.iter() {
            if a.id() < b.id() {
                assert!(
                    a_p != b_p,
                    "Overlap of {} and {} at {:?}",
                    a_s.unwrap_or(&Label::new("?")),
                    b_s.unwrap_or(&Label::new("?")),
                    a_p
                );
            }
        }
    }

    log_if_slow("check_overlap", start);
}

#[allow(dead_code)]
#[allow(clippy::needless_pass_by_value)]
pub fn check_hierarchy(
    changed: Query<
        (Entity, Option<&Pos>, Option<&Parent>, Option<&Label>),
        Or<(Changed<Pos>, Changed<Parent>)>,
    >,
) {
    let start = Instant::now();

    for (entity, pos, parent, label) in changed.iter() {
        assert!(
            pos.is_some() != parent.is_some(),
            "hierarchy violation of {} {} {}",
            label.map_or_else(
                || format!("entity {entity:?} without label"),
                |label| format!("{label}")
            ),
            parent.map_or_else(|| "without parent".to_string(), |p| format!("< {p:?}")),
            pos.map_or_else(|| "without position".to_string(), |p| format!("at {p:?}"))
        );
    }

    log_if_slow("check_hierarchy", start);
}

#[allow(dead_code)]
#[allow(clippy::needless_pass_by_value)]
pub fn check_characters(
    characters: Query<
        (
            Entity,
            Option<&Label>,
            Option<&Pos>,
            Option<&Faction>,
            Option<&Action>,
        ),
        With<Faction>,
    >,
) {
    let start = Instant::now();

    for (entity, label, pos, faction, action) in characters.iter() {
        let label = label.map_or_else(
            || format!("entity {entity:?} without label"),
            |label| format!("{label}"),
        );
        assert!(pos.is_some(), "{label} has no position");
        assert!(faction.is_some(), "{label} has no faction");
        assert!(action.is_none(), "{label} has an action");
    }

    log_if_slow("check_characters", start);
}

#[allow(clippy::needless_pass_by_value)]
pub fn check_delay(mut last_time: Local<StdInstant>) {
    let start = Instant::now();

    let delay = last_time.next();
    if Duration::new(0, 20_000_000) < delay {
        println!("Large delay: {delay:?}");
    }

    log_if_slow("check_delay", start);
}
