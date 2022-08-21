use crate::prelude::*;
use bevy::prelude::{
    BuildChildren, Commands, Component, Entity, Parent, Query, Visibility, VisibilityBundle,
};

#[derive(Component, Debug)]
pub(crate) enum Action {
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
    ExaminePos {
        target: Pos,
    },
    ExamineZoneLevel {
        target: ZoneLevel,
    },
}

impl Action {
    pub(crate) const fn step_or_stay(pos: Option<Pos>) -> Self {
        if let Some(pos) = pos {
            Self::Step { target: pos }
        } else {
            Self::Stay
        }
    }

    pub(crate) fn perform(
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
            Self::Stay => speed.stay(),
            Self::Step { target } => move_(commands, envir, actor, label, pos, target, speed),
            Self::Attack { target } => attack(commands, envir, label, pos, target, speed),
            Self::Smash { target } => smash(commands, envir, label, pos, target, speed),
            Self::Dump => dump(commands, dumpees, actor, label, pos, speed),
            Self::Pickup => pickup(
                commands,
                &mut envir.location,
                hierarchy,
                actor,
                label,
                container,
                pos,
                speed,
            ),
            Self::ExaminePos { target } => examine(commands, actor, pos, target),
            Self::ExamineZoneLevel { target } => examine(commands, actor, pos, target.base_pos()),
            Self::SwitchRunning => switch_running(commands, actor),
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
        let message = format!("can't move to {to:?}, as it is not a nbor of {from:?}");
        commands.spawn().insert(Message::error(message));
        return Milliseconds(0);
    }

    match envir.collide(from, to, true) {
        Collision::Pass => {
            commands.entity(mover).insert(to);

            if from.level != to.level {
                commands.entity(mover).insert(LevelChanged);
            }
            if Zone::from(from) != Zone::from(to) {
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
            commands.spawn().insert(Message::warn(message));
            Milliseconds(0)
        }
        Collision::Ledged => {
            let message = format!("{label} halts at the ledge");
            commands.spawn().insert(Message::warn(message));
            Milliseconds(0)
        }
        Collision::NoStairsUp => {
            let message = format!("{label} needs a stair to go up");
            commands.spawn().insert(Message::warn(message));
            Milliseconds(0)
        }
        Collision::NoStairsDown => {
            let message = format!("{label} needs a stair to go down");
            commands.spawn().insert(Message::warn(message));
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
            .insert(Message::warn(format!("{a_label} attacks nothing")));
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

    let stair_pos = Pos::new(target.x, pos.level, target.z);
    if pos.level.up() == Some(target.level) && !envir.has_stairs_up(stair_pos) {
        let message = format!("{s_label} smashes the ceiling");
        commands.spawn().insert(Message::warn(message));
        Milliseconds(0)
    } else if pos.level.down() == Some(target.level) && !envir.has_stairs_down(stair_pos) {
        let message = format!("{s_label} smashes the floor");
        commands.spawn().insert(Message::warn(message));
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
            .insert(Message::warn(format!("{s_label} smashes nothing")));
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

    if let Some((dumpee, _, dee_label)) =
        dumpees.iter().find(|(_, parent, _)| parent.get() == dumper)
    {
        commands
            .spawn()
            .insert(Message::new(format!("{dr_label} drops {dee_label}")));
        commands.entity(dumper).remove_children(&[dumpee]);
        commands
            .entity(dumpee)
            .insert_bundle(VisibilityBundle::default())
            .insert(pos);
        speed.stay()
    } else {
        commands
            .spawn()
            .insert(Message::warn(format!("nothing to drop for {dr_label}")));
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
            .filter(|(parent, _)| parent.get() == picker)
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
            commands.spawn().insert(Message::warn(message));
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

fn examine(commands: &mut Commands, player: Entity, from: Pos, to: Pos) -> Milliseconds {
    // see update_status_detais() in systems/update.rs

    if from.level != to.level {
        commands.entity(player).insert(LevelChanged);
    }
    if Zone::from(from) != Zone::from(to) {
        commands.entity(player).insert(ZoneChanged);
    }

    Milliseconds(0)
}

fn switch_running(commands: &mut Commands, switcher: Entity) -> Milliseconds {
    commands.entity(switcher).insert(Speed::from_h_kmph(11));
    Milliseconds(0)
}
