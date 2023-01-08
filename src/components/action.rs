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
    Close {
        target: Pos, // nbor pos
    },
    Pickup,
    Dump,
    SwitchRunning,
}

impl Action {
    pub(crate) fn perform(
        self,
        commands: &mut Commands,
        envir: &mut Envir,
        dumpees: &Query<(Entity, &Label, &Parent)>,
        hierarchy: &Hierarchy, // pickup
        actor: Entity,
        label: &Label,
        pos: Pos,
        speed: BaseSpeed,
        melee: &Melee,
        _hands: Option<&Hands>,
        clothing: Option<&Clothing>,
    ) -> Milliseconds {
        let duration: Milliseconds = match self {
            Self::Stay => speed.stay(),
            Self::Step { target } => move_(commands, envir, actor, label, pos, target, speed),
            Self::Attack { target } => attack(commands, envir, label, pos, target, speed, melee),
            Self::Smash { target } => smash(commands, envir, label, pos, target, speed, melee),
            Self::Close { target } => close(commands, envir, label, pos, target, speed),
            Self::Dump => dump(commands, dumpees, actor, label, pos, speed),
            Self::Pickup => pickup(
                commands,
                &mut envir.location,
                hierarchy,
                actor,
                label,
                clothing.unwrap(),
                pos,
                speed,
            ),
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
    speed: BaseSpeed,
) -> Milliseconds {
    if !envir.are_nbors(from, to) {
        let message = format!("can't move to {to:?}, as it is not a nbor of {from:?}");
        commands.spawn(Message::error(message));
        return Milliseconds(0);
    }

    match envir.collide(from, to, true) {
        Collision::Pass => {
            commands.entity(mover).insert(to);
            envir.walking_cost(from, to).duration(speed)
        }
        /*Collision::Fall(fall_pos) => {
            *pos = fall_pos;
            location.add(mover, *pos);
            VERTICAL
        }*/
        Collision::Blocked(obstacle) => {
            let message = format!("{label} crashes into {obstacle}");
            commands.spawn(Message::warn(message));
            Milliseconds(0)
        }
        Collision::Ledged => {
            let message = format!("{label} halts at the ledge");
            commands.spawn(Message::warn(message));
            Milliseconds(0)
        }
        Collision::Opened(door) => {
            commands.entity(door).insert(Toggle);
            envir.walking_cost(from, to).duration(speed)
        }
    }
}

fn attack(
    commands: &mut Commands,
    envir: &mut Envir,
    a_label: &Label,
    pos: Pos,
    target: Pos,
    speed: BaseSpeed,
    melee: &Melee,
) -> Milliseconds {
    if !envir.are_nbors(pos, target) {
        unimplemented!();
    }

    if let Some((defender, _)) = envir.find_character(target) {
        commands.entity(defender).insert(Damage {
            attacker: a_label.clone(),
            amount: melee.damage(),
        });
        envir.walking_cost(pos, target).duration(speed)
    } else {
        commands.spawn(Message::warn(format!("{a_label} attacks nothing")));
        Milliseconds(0)
    }
}

fn smash(
    commands: &mut Commands,
    envir: &mut Envir,
    s_label: &Label,
    pos: Pos,
    target: Pos,
    speed: BaseSpeed,
    melee: &Melee,
) -> Milliseconds {
    if !envir.are_nbors(pos, target) && target != pos {
        unimplemented!();
    }

    let stair_pos = Pos::new(target.x, pos.level, target.z);
    if pos.level.up() == Some(target.level) && envir.stairs_up_to(stair_pos).is_none() {
        let message = format!("{s_label} smashes the ceiling");
        commands.spawn(Message::warn(message));
        Milliseconds(0)
    } else if pos.level.down() == Some(target.level) && envir.stairs_down_to(stair_pos).is_none() {
        let message = format!("{s_label} smashes the floor");
        commands.spawn(Message::warn(message));
        Milliseconds(0)
    } else if let Some(smashable) = envir.find_item(target) {
        commands.entity(smashable).insert(Damage {
            attacker: s_label.clone(),
            amount: melee.damage(),
        });
        envir.walking_cost(pos, target).duration(speed)
    } else {
        commands.spawn(Message::warn(format!("{s_label} smashes nothing")));
        Milliseconds(0)
    }
}

fn close(
    commands: &mut Commands,
    envir: &mut Envir,
    closer: &Label,
    pos: Pos,
    target: Pos,
    speed: BaseSpeed,
) -> Milliseconds {
    if !envir.are_nbors(pos, target) && target != pos {
        unimplemented!();
    }

    if let Some(closable) = envir.find_closeable(target) {
        if let Some((_, character)) = envir.find_character(target) {
            let air = Label::new("the air");
            let obstacle = envir.find_terrain(target).unwrap_or(&air);
            commands.spawn(Message::warn(format!(
                "{closer} can't close {obstacle} on {character}"
            )));
            Milliseconds(0)
        } else {
            commands.entity(closable).insert(Toggle);
            envir.walking_cost(pos, target).duration(speed)
        }
    } else {
        let air = Label::new("the air");
        let obstacle = envir.find_terrain(target).unwrap_or(&air);
        commands.spawn(Message::warn(format!("{closer} can't close {obstacle}")));
        Milliseconds(0)
    }
}

fn dump(
    commands: &mut Commands,
    dumpees: &Query<(Entity, &Label, &Parent)>,
    dumper: Entity,
    dr_label: &Label,
    pos: Pos,
    speed: BaseSpeed,
) -> Milliseconds {
    // It seems impossible to remove something from 'Children', so we check 'Parent'.

    if let Some((dumpee, dee_label, _)) = dumpees.iter().find(|(.., parent)| parent.get() == dumper)
    {
        commands.spawn(Message::new(format!("{dr_label} drops {dee_label}")));
        commands.entity(dumper).remove_children(&[dumpee]);
        commands
            .entity(dumpee)
            .insert(VisibilityBundle::default())
            .insert(pos);
        speed.stay()
    } else {
        commands.spawn(Message::warn(format!("nothing to drop for {dr_label}")));
        Milliseconds(0)
    }
}

fn pickup(
    commands: &mut Commands,
    location: &mut Location,
    hierarchy: &Hierarchy,
    picker: Entity,
    pr_label: &Label,
    clothing: &Clothing,
    pr_pos: Pos,
    speed: BaseSpeed,
) -> Milliseconds {
    if let Some((pd_entity, pd_label, pd_containable)) =
        location.get_first(pr_pos, &hierarchy.picked)
    {
        let current_items = hierarchy
            .children
            .iter()
            .filter(|(parent, _)| parent.get() == picker)
            .map(|(_, containable)| containable);
        match clothing
            .0
            .check_add(pr_label, current_items, pd_containable, pd_label)
        {
            Ok(()) => {
                let message = format!("{pr_label} picks up {pd_label}", pd_label = &pd_label);
                commands.spawn(Message::new(message));
                commands
                    .entity(pd_entity)
                    .remove::<Pos>()
                    .remove::<Visibility>();
                commands.entity(picker).push_children(&[pd_entity]);
                speed.activate()
            }
            Err(messages) => {
                assert!(!messages.is_empty());
                commands.spawn_batch(messages);
                Milliseconds(0)
            }
        }
    } else {
        let message = format!("nothing to pick up for {pr_label}");
        commands.spawn(Message::new(message));
        Milliseconds(0)
    }
}

fn switch_running(commands: &mut Commands, switcher: Entity) -> Milliseconds {
    commands.entity(switcher).insert(BaseSpeed::from_h_kmph(11));
    Milliseconds(0)
}
