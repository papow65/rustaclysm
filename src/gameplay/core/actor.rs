use crate::prelude::*;
use bevy::prelude::*;

#[derive(Copy, Clone, Debug)]
pub(crate) enum Breath {
    Normal,
    Winded,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum StaminaImpact {
    FullRest,
    Rest,
    Light,
    Neutral,
    Heavy,
}

impl StaminaImpact {
    pub(crate) fn as_i16(&self) -> i16 {
        match self {
            Self::FullRest => 100,
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
    fn full_rest(timeout: Milliseconds) -> Self {
        Self::new(timeout, StaminaImpact::FullRest)
    }

    #[must_use]
    fn heavy(timeout: Milliseconds) -> Self {
        Self::new(timeout, StaminaImpact::Heavy)
    }
}

pub(crate) struct Actor<'s> {
    pub(crate) entity: Entity,
    pub(crate) name: &'s ObjectName,
    pub(crate) pos: Pos,
    pub(crate) base_speed: BaseSpeed,
    pub(crate) health: &'s Health,
    pub(crate) faction: &'s Faction,
    pub(crate) melee: &'s Melee,
    pub(crate) body_containers: Option<&'s BodyContainers>,
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

    pub(crate) fn stay(&self, duration: StayDuration) -> Impact {
        match duration {
            StayDuration::Short => Impact::rest(
                Millimeter(Millimeter::ADJACENT.0 / 2)
                    / self.high_speed().unwrap_or_else(|| self.speed()),
            ),
            StayDuration::Long => Impact::full_rest(Milliseconds::MINUTE),
        }
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
            commands.spawn(Message::error().push(self.name.single()).add(format!(
                "can't move to {to:?}, as it is not a nbor of {from:?}"
            )));
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
                commands.spawn(
                    Message::warn()
                        .push(self.name.single())
                        .str("crashes into")
                        .push(obstacle.single()),
                );
                None
            }
            Collision::Ledged => {
                commands.spawn(
                    Message::warn()
                        .push(self.name.single())
                        .str("halts at the ledge"),
                );
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
            commands.spawn(Message::warn().push(self.name.single()).str("is too exhausted to attack"));
            return None;
        };

        if !envir.are_nbors(self.pos, target) {
            unimplemented!();
        }

        if let Some((defender, _)) = envir.find_character(target) {
            commands.entity(defender).insert(Damage {
                attacker: self.name.single(),
                amount: self.melee.damage(),
            });
            Some(Impact::heavy(
                envir.walking_cost(self.pos, target).duration(high_speed),
            ))
        } else {
            commands.spawn(
                Message::warn()
                    .push(self.name.single())
                    .str("attacks nothing"),
            );
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
            commands.spawn(Message::warn().push(self.name.single()).str("is too exhausted to smash"));
            return None;
        };

        if !envir.are_nbors(self.pos, target) && target != self.pos {
            unimplemented!();
        }

        let stair_pos = Pos::new(target.x, self.pos.level, target.z);
        if self.pos.level.up() == Some(target.level) && envir.stairs_up_to(stair_pos).is_none() {
            commands.spawn(
                Message::warn()
                    .push(self.name.single())
                    .str("smashes the ceiling"),
            );
            None
        } else if self.pos.level.down() == Some(target.level)
            && envir.stairs_down_to(stair_pos).is_none()
        {
            commands.spawn(
                Message::warn()
                    .push(self.name.single())
                    .str("smashes the floor"),
            );
            None
        } else if let Some((smashable, _)) = envir.find_smashable(target) {
            commands.entity(smashable).insert(Damage {
                attacker: self.name.single(),
                amount: self.melee.damage(),
            });
            Some(Impact::heavy(
                envir.walking_cost(self.pos, target).duration(high_speed),
            ))
        } else {
            commands.spawn(
                Message::warn()
                    .push(self.name.single())
                    .str("smashes nothing"),
            );
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

        if let Some((closeable, closeable_name)) = envir.find_closeable(target) {
            if let Some((_, character)) = envir.find_character(target) {
                commands.spawn(
                    Message::warn()
                        .push(self.name.single())
                        .str("can't close")
                        .push(closeable_name.single())
                        .str("on")
                        .push(character.single()),
                );
                None
            } else {
                commands.entity(closeable).insert(Toggle);
                Some(
                    self.standard_impact(
                        envir.walking_cost(self.pos, target).duration(self.speed()),
                    ),
                )
            }
        } else {
            let missing = ObjectName::missing();
            let obstacle = envir.find_terrain(target).unwrap_or(&missing);
            commands.spawn(
                Message::warn()
                    .push(self.name.single())
                    .str("can't close")
                    .push(obstacle.single()),
            );
            None
        }
    }

    pub(crate) fn wield(
        &'s self,
        commands: &mut Commands,
        location: &mut Location,
        hierarchy: &Hierarchy,
    ) -> Option<Impact> {
        self.take(
            commands,
            location,
            hierarchy,
            self.body_containers.unwrap().hands,
        )
    }

    pub(crate) fn pickup(
        &'s self,
        commands: &mut Commands,
        location: &mut Location,
        hierarchy: &Hierarchy,
    ) -> Option<Impact> {
        self.take(
            commands,
            location,
            hierarchy,
            self.body_containers.unwrap().clothing,
        )
    }

    pub(crate) fn take(
        &'s self,
        commands: &mut Commands,
        location: &mut Location,
        hierarchy: &Hierarchy,
        container_entity: Entity,
    ) -> Option<Impact> {
        if let Some((
            taken_entity,
            taken_object_name,
            taken_amount,
            taken_filthy,
            taken_containable,
        )) = location.get_first(self.pos, &hierarchy.picked)
        {
            let current_items = hierarchy
                .children
                .iter()
                .filter(|(parent, _)| parent.get() == container_entity)
                .map(|(_, containable)| containable);

            let container = hierarchy.containers.get(container_entity).unwrap();
            let taken_name = taken_object_name.as_item(taken_amount, taken_filthy);
            match container.check_add(
                self.name.single(),
                current_items,
                taken_containable,
                taken_name.clone(),
            ) {
                Ok(()) => {
                    commands.spawn(
                        Message::info()
                            .push(self.name.single())
                            .str("picks up")
                            .extend(taken_name),
                    );
                    commands
                        .entity(container_entity)
                        .push_children(&[taken_entity]);
                    commands.entity(taken_entity).remove::<Pos>();
                    location.update(taken_entity, None);
                    Some(self.activate())
                }
                Err(messages) => {
                    assert!(!messages.is_empty());
                    commands.spawn_batch(messages);
                    None
                }
            }
        } else {
            commands.spawn(
                Message::warn()
                    .str("Nothing to pick up for")
                    .push(self.name.single()),
            );
            None
        }
    }

    pub(crate) fn dump(
        &'s self,
        commands: &mut Commands,
        location: &mut Location,
        dumpees: &Query<(
            Entity,
            &ObjectName,
            Option<&Amount>,
            Option<&Filthy>,
            &Parent,
        )>,
    ) -> Option<Impact> {
        // It seems impossible to remove something from 'Children', so we check 'Parent'.

        for container in [
            self.body_containers.unwrap().clothing,
            self.body_containers.unwrap().hands,
        ] {
            if let Some((dumpee, dee_name, dee_amount, dee_filthy, _)) = dumpees
                .iter()
                .find(|(.., parent)| parent.get() == container)
            {
                commands.spawn(
                    Message::info()
                        .push(self.name.single())
                        .str("drops")
                        .extend(dee_name.as_item(dee_amount, dee_filthy)),
                );
                commands.entity(container).remove_children(&[dumpee]);
                commands
                    .entity(dumpee)
                    .insert(VisibilityBundle::default())
                    .insert(self.pos);
                location.update(dumpee, Some(self.pos));
                return Some(self.activate());
            }
        }

        commands.spawn(
            Message::warn()
                .str("nothing to drop up for")
                .push(self.name.single()),
        );
        None
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
    &'s ObjectName,
    &'s Pos,
    &'s BaseSpeed,
    &'s Health,
    &'s Faction,
    &'s Melee,
    Option<&'s BodyContainers>,
    Option<&'s Aquatic>,
    Option<&'s LastEnemy>,
    &'s Stamina,
    &'s WalkingMode,
);

impl<'s> From<ActorTuple<'s>> for Actor<'s> {
    fn from(
        (
            entity,
            name,
            &pos,
            &base_speed,
            health,
            faction,
            melee,
            body_containers,
            aquatic,
            last_enemy,
            stamina,
            walking_mode,
        ): ActorTuple<'s>,
    ) -> Self {
        Self {
            entity,
            name,
            pos,
            base_speed,
            health,
            faction,
            melee,
            body_containers,
            aquatic,
            last_enemy,
            stamina,
            walking_mode,
        }
    }
}
