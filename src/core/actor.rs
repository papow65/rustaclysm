use crate::prelude::*;
use bevy::prelude::{BuildChildren, Commands, Entity, Parent, Query, Visibility, VisibilityBundle};

pub(crate) struct Actor<'s> {
    pub(crate) entity: Entity,
    pub(crate) label: &'s Label,
    pub(crate) pos: Pos,
    pub(crate) speed: BaseSpeed,
    pub(crate) health: &'s Health,
    pub(crate) faction: &'s Faction,
    pub(crate) melee: &'s Melee,
    pub(crate) hands: Option<&'s Hands>,
    pub(crate) clothing: Option<&'s Clothing>,
    pub(crate) aquatic: Option<&'s Aquatic>,
    pub(crate) last_enemy: Option<&'s LastEnemy>,
}

impl<'s> Actor<'s> {
    pub(crate) fn perform(
        &'s self,
        commands: &mut Commands,
        envir: &mut Envir,
        dumpees: &Query<(Entity, &Label, &Parent)>,
        hierarchy: &Hierarchy, // pickup
        action: &Action,
    ) -> Milliseconds {
        let duration: Milliseconds = match action {
            Action::Stay => self.stay(),
            Action::Step { target } => self.move_(commands, envir, *target),
            Action::Attack { target } => self.attack(commands, envir, *target),
            Action::Smash { target } => self.smash(commands, envir, *target),
            Action::Close { target } => self.close(commands, envir, *target),
            Action::Wield => self.wield(commands, &mut envir.location, hierarchy),
            Action::Pickup => self.pickup(commands, &mut envir.location, hierarchy),
            Action::Dump => self.dump(commands, dumpees),
            Action::SwitchRunning => self.switch_running(commands),
        };

        //println!("removing finished action: {action:?}");
        /*commands.entity(actor).remove::<Action>();
                *        if 0 < duration.0 {
                *            commands.entity(actor).insert(Timeout {
                *                until: game_progress.ms + duration,
        });
        }*/
        duration
    }

    fn stay(&'s self) -> Milliseconds {
        self.speed.stay()
    }

    fn move_(&'s self, commands: &mut Commands, envir: &mut Envir, to: Pos) -> Milliseconds {
        let from = self.pos;
        if !envir.are_nbors(self.pos, to) {
            let message = format!("can't move to {to:?}, as it is not a nbor of {from:?}");
            commands.spawn(Message::error(message));
            return Milliseconds(0);
        }

        match envir.collide(from, to, true) {
            Collision::Pass => {
                commands.entity(self.entity).insert(to);
                envir.walking_cost(from, to).duration(self.speed)
            }
            /*Collision::Fall(fall_pos) => {
                 * pos = fall_pos;
                 *            location.add(mover, *pos);
                 *            VERTICAL
            }*/
            Collision::Blocked(obstacle) => {
                let message = format!("{} crashes into {obstacle}", self.label);
                commands.spawn(Message::warn(message));
                Milliseconds(0)
            }
            Collision::Ledged => {
                let message = format!("{} halts at the ledge", self.label);
                commands.spawn(Message::warn(message));
                Milliseconds(0)
            }
            Collision::Opened(door) => {
                commands.entity(door).insert(Toggle);
                envir.walking_cost(from, to).duration(self.speed)
            }
        }
    }

    fn attack(&'s self, commands: &mut Commands, envir: &mut Envir, target: Pos) -> Milliseconds {
        if !envir.are_nbors(self.pos, target) {
            unimplemented!();
        }

        if let Some((defender, _)) = envir.find_character(target) {
            commands.entity(defender).insert(Damage {
                attacker: self.label.clone(),
                amount: self.melee.damage(),
            });
            envir.walking_cost(self.pos, target).duration(self.speed)
        } else {
            commands.spawn(Message::warn(format!("{} attacks nothing", self.label)));
            Milliseconds(0)
        }
    }

    fn smash(&'s self, commands: &mut Commands, envir: &mut Envir, target: Pos) -> Milliseconds {
        if !envir.are_nbors(self.pos, target) && target != self.pos {
            unimplemented!();
        }

        let stair_pos = Pos::new(target.x, self.pos.level, target.z);
        if self.pos.level.up() == Some(target.level) && envir.stairs_up_to(stair_pos).is_none() {
            let message = format!("{} smashes the ceiling", self.label);
            commands.spawn(Message::warn(message));
            Milliseconds(0)
        } else if self.pos.level.down() == Some(target.level)
            && envir.stairs_down_to(stair_pos).is_none()
        {
            let message = format!("{} smashes the floor", self.label);
            commands.spawn(Message::warn(message));
            Milliseconds(0)
        } else if let Some(smashable) = envir.find_item(target) {
            commands.entity(smashable).insert(Damage {
                attacker: self.label.clone(),
                amount: self.melee.damage(),
            });
            envir.walking_cost(self.pos, target).duration(self.speed)
        } else {
            commands.spawn(Message::warn(format!("{} smashes nothing", self.label)));
            Milliseconds(0)
        }
    }

    fn close(&'s self, commands: &mut Commands, envir: &mut Envir, target: Pos) -> Milliseconds {
        if !envir.are_nbors(self.pos, target) && target != self.pos {
            unimplemented!();
        }

        if let Some(closable) = envir.find_closeable(target) {
            if let Some((_, character)) = envir.find_character(target) {
                let air = Label::new("the air");
                let obstacle = envir.find_terrain(target).unwrap_or(&air);
                commands.spawn(Message::warn(format!(
                    "{} can't close {obstacle} on {character}",
                    self.label
                )));
                Milliseconds(0)
            } else {
                commands.entity(closable).insert(Toggle);
                envir.walking_cost(self.pos, target).duration(self.speed)
            }
        } else {
            let air = Label::new("the air");
            let obstacle = envir.find_terrain(target).unwrap_or(&air);
            commands.spawn(Message::warn(format!(
                "{} can't close {obstacle}",
                self.label
            )));
            Milliseconds(0)
        }
    }

    fn wield(
        &'s self,
        commands: &mut Commands,
        location: &mut Location,
        hierarchy: &Hierarchy,
    ) -> Milliseconds {
        self.take(commands, location, hierarchy, &self.hands.unwrap().0)
    }

    fn pickup(
        &'s self,
        commands: &mut Commands,
        location: &mut Location,
        hierarchy: &Hierarchy,
    ) -> Milliseconds {
        self.take(commands, location, hierarchy, &self.clothing.unwrap().0)
    }

    fn take(
        &'s self,
        commands: &mut Commands,
        location: &mut Location,
        hierarchy: &Hierarchy,
        container: &Container,
    ) -> Milliseconds {
        if let Some((pd_entity, pd_label, pd_containable)) =
            location.get_first(self.pos, &hierarchy.picked)
        {
            let current_items = hierarchy
                .children
                .iter()
                .filter(|(parent, _)| parent.get() == self.entity)
                .map(|(_, containable)| containable);
            match container.check_add(self.label, current_items, pd_containable, pd_label) {
                Ok(()) => {
                    let message = format!("{} picks up {}", self.label, &pd_label);
                    commands.spawn(Message::new(message));
                    commands
                        .entity(pd_entity)
                        .remove::<Pos>()
                        .remove::<Visibility>();
                    commands.entity(self.entity).push_children(&[pd_entity]);
                    self.speed.activate()
                }
                Err(messages) => {
                    assert!(!messages.is_empty());
                    commands.spawn_batch(messages);
                    Milliseconds(0)
                }
            }
        } else {
            let message = format!("nothing to pick up for {}", self.label);
            commands.spawn(Message::warn(message));
            Milliseconds(0)
        }
    }

    fn dump(
        &'s self,
        commands: &mut Commands,
        dumpees: &Query<(Entity, &Label, &Parent)>,
    ) -> Milliseconds {
        // It seems impossible to remove something from 'Children', so we check 'Parent'.

        if let Some((dumpee, dee_label, _)) = dumpees
            .iter()
            .find(|(.., parent)| parent.get() == self.entity)
        {
            commands.spawn(Message::new(format!("{} drops {dee_label}", self.label)));
            commands.entity(self.entity).remove_children(&[dumpee]);
            commands
                .entity(dumpee)
                .insert(VisibilityBundle::default())
                .insert(self.pos);
            self.speed.stay()
        } else {
            commands.spawn(Message::warn(format!("nothing to drop for {}", self.label)));
            Milliseconds(0)
        }
    }

    fn switch_running(&'s self, commands: &mut Commands) -> Milliseconds {
        commands
            .entity(self.entity)
            .insert(BaseSpeed::from_h_kmph(11));
        Milliseconds(0)
    }
}

pub(crate) type ActorTuple<'s> = (
    Entity,
    &'s Label,
    &'s Pos,
    &'s BaseSpeed,
    &'s Health,
    &'s Faction,
    &'s Melee,
    Option<&'s Hands>,
    Option<&'s Clothing>,
    Option<&'s Aquatic>,
    Option<&'s LastEnemy>,
);

impl<'s> From<ActorTuple<'s>> for Actor<'s> {
    fn from(
        (entity, label, &pos, &speed, health, faction, melee, hands, clothing, aquatic, last_enemy): ActorTuple<'s>,
    ) -> Self {
        Self {
            entity,
            label,
            pos,
            speed,
            health,
            faction,
            melee,
            hands,
            clothing,
            aquatic,
            last_enemy,
        }
    }
}
