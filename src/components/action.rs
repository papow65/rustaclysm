use bevy::prelude::*;

use super::super::resources::{Collision, Envir, Hierarchy, Location};
use super::super::unit::{Milliseconds, Speed};
use super::{Container, Damage, Label, Message, Pos, PosYChanged, ZoneChanged};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Offset(Pos),
    Pickup,
    Dump,
    Attack,
    Smash,
    SwitchRunning,
    SwitchExamining,
    Cancel,
}

impl Instruction {
    pub const fn new(key_code: KeyCode) -> Option<Self> {
        Some(match key_code {
            KeyCode::Numpad1
            | KeyCode::Numpad2
            | KeyCode::Numpad3
            | KeyCode::Numpad4
            | KeyCode::Numpad5
            | KeyCode::Numpad6
            | KeyCode::Numpad7
            | KeyCode::Numpad8
            | KeyCode::Numpad9
            | KeyCode::R
            | KeyCode::F => Self::Offset(Self::get_relative_pos(key_code)),
            KeyCode::B => Self::Pickup,
            KeyCode::V => Self::Dump,
            KeyCode::A => Self::Attack,
            KeyCode::S => Self::Smash,
            KeyCode::X => Self::SwitchExamining,
            KeyCode::NumpadAdd => Self::SwitchRunning,
            KeyCode::Escape => Self::Cancel,
            _ => {
                return None;
            }
        })
    }

    const fn get_relative_pos(key_code: KeyCode) -> Pos {
        Pos(
            match key_code {
                KeyCode::Numpad1 | KeyCode::Numpad2 | KeyCode::Numpad3 => -1,
                KeyCode::Numpad7 | KeyCode::Numpad8 | KeyCode::Numpad9 => 1,
                _ => 0,
            },
            match key_code {
                KeyCode::R => 1,
                KeyCode::F => -1,
                _ => 0,
            },
            match key_code {
                KeyCode::Numpad1 | KeyCode::Numpad4 | KeyCode::Numpad7 => -1,
                KeyCode::Numpad3 | KeyCode::Numpad6 | KeyCode::Numpad9 => 1,
                _ => 0,
            },
        )
    }
}

#[derive(Component, Debug)]
pub enum Action {
    Stay,
    Step {
        target: Pos, // nbor pos
    },
    Attack {
        target: Pos, // nbor pos
    },
    Smash {
        target: Pos, // nbor pos
    },
    Pickup,
    Dump,
    SwitchRunning,
    Examine {
        target: Pos,
    },
}

impl Action {
    pub const fn step_or_stay(pos: Option<Pos>) -> Self {
        if let Some(pos) = pos {
            Self::Step { target: pos }
        } else {
            Self::Stay
        }
    }

    pub fn perform(
        self,
        commands: &mut Commands,
        envir: &mut Envir,
        dumpees: &Query<(Entity, &Parent, &Label)>,
        hierarchy: &Hierarchy, // pickup
        actor: Entity,
        label: &Label,
        pos: Pos,
        speed: Speed,
        container: &Container,
    ) -> Milliseconds {
        let duration: Milliseconds = match self {
            Action::Stay => speed.stay(),
            Action::Step { target } => move_(commands, envir, actor, label, pos, target, speed),
            Action::Attack { target } => attack(commands, envir, label, pos, target, speed),
            Action::Smash { target } => smash(commands, envir, label, pos, target, speed),
            Action::Dump => dump(commands, dumpees, actor, label, pos, speed),
            Action::Pickup => pickup(
                commands,
                &mut envir.location,
                hierarchy,
                actor,
                label,
                container,
                pos,
                speed,
            ),
            Action::Examine { target: _ } => examine(),
            Action::SwitchRunning => switch_running(commands, actor),
        };

        //println!("removing finished action: {action:?}");
        /*commands.entity(actor).remove::<Action>();
        if 0 < duration.0 {
            commands.entity(actor).insert(Timeout {
                until: game_progress.ms + duration,
            });
        }*/
        duration
    }
}

fn move_(
    commands: &mut Commands,
    envir: &mut Envir,
    mover: Entity,
    label: &Label,
    from: Pos,
    to: Pos,
    speed: Speed,
) -> Milliseconds {
    if !to.is_potential_nbor(from) {
        let message = format!("STEP ERROR: {to:?} is not a nbor of {from:?}");
        commands.spawn().insert(Message::new(message));
        return Milliseconds(0);
    }

    match envir.collide(from, to, true) {
        Collision::Pass => {
            commands.entity(mover).insert(to);

            if from.1 != to.1 {
                commands.entity(mover).insert(PosYChanged);
            }

            if from.0.div_euclid(24) != to.0.div_euclid(24)
                || from.2.div_euclid(24) != to.2.div_euclid(24)
            {
                commands.spawn().insert(Message::new("Zone changed"));
                commands.entity(mover).insert(ZoneChanged);
            }

            from.dist(to) / speed
        }
        /*Collision::Fall(fall_pos) => {
            *pos = fall_pos;
            location.add(mover, *pos);
            VERTICAL
        }*/
        Collision::Blocked(obstacle) => {
            let message = format!("{label} crashes into {obstacle}");
            commands.spawn().insert(Message::new(message));
            Milliseconds(0)
        }
        Collision::Ledged => {
            let message = format!("{label} halts at the ledge");
            commands.spawn().insert(Message::new(message));
            Milliseconds(0)
        }
        Collision::NoStairsUp => {
            let message = format!("{label} needs a stair to go up");
            commands.spawn().insert(Message::new(message));
            Milliseconds(0)
        }
        Collision::NoStairsDown => {
            let message = format!("{label} needs a stair to go down");
            commands.spawn().insert(Message::new(message));
            Milliseconds(0)
        }
    }
}

fn attack(
    commands: &mut Commands,
    envir: &mut Envir,
    a_label: &Label,
    pos: Pos,
    target: Pos,
    speed: Speed,
) -> Milliseconds {
    if !target.is_potential_nbor(pos) {
        unimplemented!();
    }

    if let Some(defender) = envir.find_character(target) {
        commands.entity(defender).insert(Damage {
            attacker: a_label.clone(),
            amount: 1,
        });
        pos.dist(target) / speed
    } else {
        commands
            .spawn()
            .insert(Message::new(format!("{a_label} attacks nothing")));
        Milliseconds(0)
    }
}

fn smash(
    commands: &mut Commands,
    envir: &mut Envir,
    s_label: &Label,
    pos: Pos,
    target: Pos,
    speed: Speed,
) -> Milliseconds {
    if !target.is_potential_nbor(pos) && target != pos {
        unimplemented!();
    }

    let stair_pos = Pos(target.0, pos.1, target.2);
    if pos.1 < target.1 && !envir.has_stairs_up(stair_pos) {
        let message = format!("{s_label} smashes the ceiling");
        commands.spawn().insert(Message::new(message));
        Milliseconds(0)
    } else if target.1 < pos.1 && !envir.has_stairs_down(stair_pos) {
        let message = format!("{s_label} smashes the floor");
        commands.spawn().insert(Message::new(message));
        Milliseconds(0)
    } else if let Some(smashable) = envir.find_item(target) {
        commands.entity(smashable).insert(Damage {
            attacker: s_label.clone(),
            amount: 1,
        });
        pos.dist(target) / speed
    } else {
        commands
            .spawn()
            .insert(Message::new(format!("{s_label} smashes nothing")));
        Milliseconds(0)
    }
}

fn dump(
    commands: &mut Commands,
    dumpees: &Query<(Entity, &Parent, &Label)>,
    dumper: Entity,
    dr_label: &Label,
    pos: Pos,
    speed: Speed,
) -> Milliseconds {
    // It seems impossible to remove something from 'Children', so we check 'Parent'.

    if let Some((dumpee, _, dee_label)) = dumpees
        .iter()
        .find(|(_, &Parent(parent), _)| parent == dumper)
    {
        commands
            .spawn()
            .insert(Message::new(format!("{dr_label} drops {dee_label}")));
        commands
            .entity(dumpee)
            .insert(pos)
            .insert(Visibility::default())
            .remove::<Parent>(); // Doesn't seem to affect 'Children'
        speed.stay()
    } else {
        commands
            .spawn()
            .insert(Message::new(format!("nothing to drop for {dr_label}")));
        Milliseconds(0)
    }
}

fn pickup(
    commands: &mut Commands,
    location: &mut Location,
    hierarchy: &Hierarchy,
    picker: Entity,
    pr_label: &Label,
    container: &Container,
    pr_pos: Pos,
    speed: Speed,
) -> Milliseconds {
    if let Some((pd_entity, pd_label, pd_containable)) =
        location.get_first(pr_pos, &hierarchy.picked)
    {
        let space_used: u8 = hierarchy
            .children
            .iter()
            .filter(|(&parent, _)| parent.0 == picker)
            .map(|(_, containable)| containable.0)
            .sum();
        if container.0 < space_used + pd_containable.0 {
            let message = format!(
                "{} has only {} space left, but {} needed to pick up {}",
                pr_label,
                container.0 - space_used,
                pd_containable.0,
                &pd_label
            );
            commands.spawn().insert(Message::new(message));
            Milliseconds(0)
        } else {
            let message = format!("{pr_label} picks up {pd_label}", pd_label = &pd_label);
            commands.spawn().insert(Message::new(message));
            commands
                .entity(pd_entity)
                .remove::<Pos>()
                .remove::<Visibility>();
            commands.entity(picker).push_children(&[pd_entity]);
            speed.activate()
        }
    } else {
        let message = format!("nothing to pick up for {pr_label}");
        commands.spawn().insert(Message::new(message));
        Milliseconds(0)
    }
}

const fn examine() -> Milliseconds {
    // see update_status_detais() in systems/update.rs
    Milliseconds(0)
}

fn switch_running(commands: &mut Commands, switcher: Entity) -> Milliseconds {
    commands.entity(switcher).insert(Speed::from_h_kmph(11));
    Milliseconds(0)
}
