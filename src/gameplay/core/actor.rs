use crate::prelude::*;
use bevy::prelude::*;

#[derive(Copy, Clone, Debug)]
pub(crate) enum Breath {
    Normal,
    Winded,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum StaminaImpact {
    Rest,
    Light,
    Neutral,
    Heavy,
}

impl StaminaImpact {
    pub(crate) fn as_i16(&self) -> i16 {
        match self {
            Self::Rest => 2,
            Self::Light => 1,
            Self::Neutral => 0,
            Self::Heavy => -12,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Impact {
    pub(crate) timeout: Milliseconds,
    pub(crate) stamina_impact: StaminaImpact,
}

impl Impact {
    #[must_use]
    fn new(timeout: Milliseconds, stamina_impact: StaminaImpact) -> Self {
        Self {
            timeout,
            stamina_impact,
        }
    }

    #[must_use]
    fn rest(timeout: Milliseconds) -> Self {
        Self::new(timeout, StaminaImpact::Rest)
    }

    #[must_use]
    fn heavy(timeout: Milliseconds) -> Self {
        Self::new(timeout, StaminaImpact::Heavy)
    }
}

pub(crate) struct Actor<'s> {
    pub(crate) entity: Entity,
    pub(crate) label: &'s TextLabel,
    pub(crate) pos: Pos,
    pub(crate) base_speed: BaseSpeed,
    pub(crate) health: &'s Health,
    pub(crate) faction: &'s Faction,
    pub(crate) melee: &'s Melee,
    pub(crate) hands: Option<&'s Hands>,
    pub(crate) clothing: Option<&'s Clothing>,
    pub(crate) aquatic: Option<&'s Aquatic>,
    pub(crate) last_enemy: Option<&'s LastEnemy>,
    pub(crate) stamina: &'s Stamina,
    pub(crate) walking_mode: &'s WalkingMode,
}

impl<'s> Actor<'s> {
    pub(crate) fn speed(&'s self) -> MillimeterPerSecond {
        self.base_speed
            .speed(self.walking_mode, self.stamina.breath())
    }

    fn high_speed(&'s self) -> Option<MillimeterPerSecond> {
        match self.stamina.breath() {
            Breath::Normal => Some(self.base_speed.speed(&WalkingMode::Running, Breath::Normal)),
            Breath::Winded => None,
        }
    }

    fn standard_impact(&self, timeout: Milliseconds) -> Impact {
        Impact {
            timeout,
            stamina_impact: self.walking_mode.stamina_impact(self.stamina.breath()),
        }
    }

    pub(crate) fn stay(&self) -> Impact {
        Impact::rest(
            Millimeter(Millimeter::ADJACENT.0 / 2) / self.high_speed().unwrap_or(self.speed()),
        )
    }

    fn activate(&self) -> Impact {
        self.standard_impact(Millimeter(3 * Millimeter::ADJACENT.0) / self.speed())
    }

    pub(crate) fn move_(
        &'s self,
        commands: &mut Commands,
        envir: &mut Envir,
        to: Pos,
    ) -> Option<Impact> {
        let from = self.pos;
        if !envir.are_nbors(self.pos, to) {
            let message = format!("can't move to {to:?}, as it is not a nbor of {from:?}");
            commands.spawn(Message::error(message));
            return None;
        }

        match envir.collide(from, to, true) {
            Collision::Pass => {
                commands.entity(self.entity).insert(to);
                envir.location.update(self.entity, Some(to));
                Some(self.standard_impact(envir.walking_cost(from, to).duration(self.speed())))
            }
            /*Collision::Fall(fall_pos) => {
                 * pos = fall_pos;
                 *            location.add(mover, *pos);
                 *            VERTICAL
            }*/
            Collision::Blocked(obstacle) => {
                let message = format!("{} crashes into {obstacle}", self.label);
                commands.spawn(Message::warn(message));
                None
            }
            Collision::Ledged => {
                let message = format!("{} halts at the ledge", self.label);
                commands.spawn(Message::warn(message));
                None
            }
            Collision::Opened(door) => {
                commands.entity(door).insert(Toggle);
                Some(self.standard_impact(envir.walking_cost(from, to).duration(self.speed())))
            }
        }
    }

    pub(crate) fn attack(
        &'s self,
        commands: &mut Commands,
        envir: &mut Envir,
        target: Pos,
    ) -> Option<Impact> {
        let Some(high_speed) = self.high_speed() else {
            commands.spawn(Message::warn(format!("{} is too exhausted to attack", self.label)));
            return None;
        };

        if !envir.are_nbors(self.pos, target) {
            unimplemented!();
        }

        if let Some((defender, _)) = envir.find_character(target) {
            commands.entity(defender).insert(Damage {
                attacker: self.label.clone(),
                amount: self.melee.damage(),
            });
            Some(Impact::heavy(
                envir.walking_cost(self.pos, target).duration(high_speed),
            ))
        } else {
            commands.spawn(Message::warn(format!("{} attacks nothing", self.label)));
            None
        }
    }

    pub(crate) fn smash(
        &'s self,
        commands: &mut Commands,
        envir: &mut Envir,
        target: Pos,
    ) -> Option<Impact> {
        let Some(high_speed) = self.high_speed() else {
            commands.spawn(Message::warn(format!("{} is too exhausted to smash", self.label)));
            return None;
        };

        if !envir.are_nbors(self.pos, target) && target != self.pos {
            unimplemented!();
        }

        let stair_pos = Pos::new(target.x, self.pos.level, target.z);
        if self.pos.level.up() == Some(target.level) && envir.stairs_up_to(stair_pos).is_none() {
            let message = format!("{} smashes the ceiling", self.label);
            commands.spawn(Message::warn(message));
            None
        } else if self.pos.level.down() == Some(target.level)
            && envir.stairs_down_to(stair_pos).is_none()
        {
            let message = format!("{} smashes the floor", self.label);
            commands.spawn(Message::warn(message));
            None
        } else if let Some(smashable) = envir.find_item(target) {
            commands.entity(smashable).insert(Damage {
                attacker: self.label.clone(),
                amount: self.melee.damage(),
            });
            Some(Impact::heavy(
                envir.walking_cost(self.pos, target).duration(high_speed),
            ))
        } else {
            commands.spawn(Message::warn(format!("{} smashes nothing", self.label)));
            None
        }
    }

    pub(crate) fn close(
        &'s self,
        commands: &mut Commands,
        envir: &mut Envir,
        target: Pos,
    ) -> Option<Impact> {
        if !envir.are_nbors(self.pos, target) && target != self.pos {
            unimplemented!();
        }

        if let Some(closable) = envir.find_closeable(target) {
            if let Some((_, character)) = envir.find_character(target) {
                let air = TextLabel::new("the air");
                let obstacle = envir.find_terrain(target).unwrap_or(&air);
                commands.spawn(Message::warn(format!(
                    "{} can't close {obstacle} on {character}",
                    self.label
                )));
                None
            } else {
                commands.entity(closable).insert(Toggle);
                Some(
                    self.standard_impact(
                        envir.walking_cost(self.pos, target).duration(self.speed()),
                    ),
                )
            }
        } else {
            let air = TextLabel::new("the air");
            let obstacle = envir.find_terrain(target).unwrap_or(&air);
            commands.spawn(Message::warn(format!(
                "{} can't close {obstacle}",
                self.label
            )));
            None
        }
    }

    pub(crate) fn wield(
        &'s self,
        commands: &mut Commands,
        location: &mut Location,
        hierarchy: &Hierarchy,
    ) -> Option<Impact> {
        self.take(commands, location, hierarchy, &self.hands.unwrap().0)
    }

    pub(crate) fn pickup(
        &'s self,
        commands: &mut Commands,
        location: &mut Location,
        hierarchy: &Hierarchy,
    ) -> Option<Impact> {
        self.take(commands, location, hierarchy, &self.clothing.unwrap().0)
    }

    pub(crate) fn take(
        &'s self,
        commands: &mut Commands,
        location: &mut Location,
        hierarchy: &Hierarchy,
        container: &Container,
    ) -> Option<Impact> {
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
                    commands.spawn(Message::info(message));
                    commands
                        .entity(pd_entity)
                        .remove::<Pos>()
                        .remove::<Visibility>();
                    commands.entity(self.entity).push_children(&[pd_entity]);
                    location.update(pd_entity, None);
                    Some(self.activate())
                }
                Err(messages) => {
                    assert!(!messages.is_empty());
                    commands.spawn_batch(messages);
                    None
                }
            }
        } else {
            let message = format!("nothing to pick up for {}", self.label);
            commands.spawn(Message::warn(message));
            None
        }
    }

    pub(crate) fn dump(
        &'s self,
        commands: &mut Commands,
        location: &mut Location,
        dumpees: &Query<(Entity, &TextLabel, &Parent)>,
    ) -> Option<Impact> {
        // It seems impossible to remove something from 'Children', so we check 'Parent'.

        if let Some((dumpee, dee_label, _)) = dumpees
            .iter()
            .find(|(.., parent)| parent.get() == self.entity)
        {
            commands.spawn(Message::info(format!("{} drops {dee_label}", self.label)));
            commands.entity(self.entity).remove_children(&[dumpee]);
            commands
                .entity(dumpee)
                .insert(VisibilityBundle::default())
                .insert(self.pos);
            location.update(dumpee, Some(self.pos));
            Some(self.stay())
        } else {
            commands.spawn(Message::warn(format!("nothing to drop for {}", self.label)));
            None
        }
    }

    pub(crate) fn switch_running(&'s self, commands: &mut Commands) -> Option<Impact> {
        commands
            .entity(self.entity)
            .insert(self.walking_mode.switch());
        None
    }
}

pub(crate) type ActorTuple<'s> = (
    Entity,
    &'s TextLabel,
    &'s Pos,
    &'s BaseSpeed,
    &'s Health,
    &'s Faction,
    &'s Melee,
    Option<&'s Hands>,
    Option<&'s Clothing>,
    Option<&'s Aquatic>,
    Option<&'s LastEnemy>,
    &'s Stamina,
    &'s WalkingMode,
);

impl<'s> From<ActorTuple<'s>> for Actor<'s> {
    fn from(
        (
            entity,
            label,
            &pos,
            &base_speed,
            health,
            faction,
            melee,
            hands,
            clothing,
            aquatic,
            last_enemy,
            stamina,
            walking_mode,
        ): ActorTuple<'s>,
    ) -> Self {
        Self {
            entity,
            label,
            pos,
            base_speed,
            health,
            faction,
            melee,
            hands,
            clothing,
            aquatic,
            last_enemy,
            stamina,
            walking_mode,
        }
    }
}
