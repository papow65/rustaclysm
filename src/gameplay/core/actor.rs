use crate::prelude::*;
use bevy::{ecs::query::WorldQuery, prelude::*};
use either::Either;

/** Derived from stamina */
#[derive(Copy, Clone, Debug)]
pub(crate) enum Breath {
    Normal,
    Winded,
}

#[must_use]
#[derive(Debug)]
pub(crate) struct Impact {
    pub(crate) actor_entity: Entity,
    pub(crate) timeout: Milliseconds,
    pub(crate) stamina_impact: StaminaImpact,
}

impl Impact {
    const fn new(
        actor_entity: Entity,
        timeout: Milliseconds,
        stamina_impact: StaminaImpact,
    ) -> Self {
        Self {
            actor_entity,
            timeout,
            stamina_impact,
        }
    }

    const fn rest(actor_entity: Entity, timeout: Milliseconds) -> Self {
        Self::new(actor_entity, timeout, StaminaImpact::Rest)
    }

    const fn full_rest(actor_entity: Entity, timeout: Milliseconds) -> Self {
        Self::new(actor_entity, timeout, StaminaImpact::FullRest)
    }

    const fn heavy(actor_entity: Entity, timeout: Milliseconds) -> Self {
        Self::new(actor_entity, timeout, StaminaImpact::Heavy)
    }
}

#[derive(WorldQuery)]
pub(crate) struct Actor {
    pub(crate) entity: Entity,
    pub(crate) name: &'static ObjectName,
    pub(crate) pos: &'static Pos,
    pub(crate) base_speed: &'static BaseSpeed,
    pub(crate) health: &'static Health,
    pub(crate) faction: &'static Faction,
    pub(crate) melee: &'static Melee,
    pub(crate) body_containers: Option<&'static BodyContainers>,
    pub(crate) aquatic: Option<&'static Aquatic>,
    pub(crate) last_enemy: Option<&'static LastEnemy>,
    pub(crate) stamina: &'static Stamina,
    pub(crate) walking_mode: &'static WalkingMode,
    pub(crate) life: &'static Life,
}

impl ActorItem<'_> {
    pub(crate) fn speed(&self) -> MillimeterPerSecond {
        self.base_speed
            .speed(self.walking_mode, self.stamina.breath())
    }

    fn high_speed(&self) -> Option<MillimeterPerSecond> {
        match self.stamina.breath() {
            Breath::Normal => Some(self.base_speed.speed(&WalkingMode::Running, Breath::Normal)),
            Breath::Winded => None,
        }
    }

    const fn standard_impact(&self, timeout: Milliseconds) -> Impact {
        Impact {
            actor_entity: self.entity,
            timeout,
            stamina_impact: self.walking_mode.stamina_impact(self.stamina.breath()),
        }
    }

    pub(crate) fn stay(&self, stay: &Stay) -> Impact {
        match stay.duration {
            StayDuration::Short => Impact::rest(
                self.entity,
                Millimeter(Millimeter::ADJACENT.0 / 2)
                    / self.high_speed().unwrap_or_else(|| self.speed()),
            ),
            StayDuration::Long => Impact::full_rest(self.entity, Milliseconds::MINUTE),
        }
    }

    fn activate(&self) -> Impact {
        self.standard_impact(Millimeter(3 * Millimeter::ADJACENT.0) / self.speed())
    }

    pub(crate) fn step(
        &self,
        commands: &mut Commands,
        message_writer: &mut EventWriter<Message>,
        toggle_writer: &mut EventWriter<TerrainEvent<Toggle>>,
        envir: &mut Envir,
        step: &Step,
    ) -> Option<Impact> {
        let from = *self.pos;
        if !envir.are_nbors(*self.pos, step.to) {
            message_writer.send(Message::error(Phrase::from_name(self.name).add(format!(
                "can't move to {:?}, as it is not a nbor of {from:?}",
                step.to
            ))));
            return None;
        }

        match envir.collide(from, step.to, true) {
            Collision::Pass => {
                commands.entity(self.entity).insert(step.to);
                envir.location.update(self.entity, Some(step.to));
                Some(self.standard_impact(envir.walking_cost(from, step.to).duration(self.speed())))
            }
            /*Collision::Fall(fall_pos) => {
                 * pos = fall_pos;
                 *            location.add(mover, *pos);
                 *            VERTICAL
            }*/
            Collision::Blocked(obstacle) => {
                message_writer.send(Message::warn(
                    Phrase::from_name(self.name)
                        .add("crashes into")
                        .push(obstacle.single()),
                ));
                None
            }
            Collision::Ledged => {
                message_writer.send(Message::warn(
                    Phrase::from_name(self.name).add("halts at the ledge"),
                ));
                None
            }
            Collision::Opened(door) => {
                toggle_writer.send(TerrainEvent {
                    terrain_entity: door,
                    change: Toggle::Open,
                });
                Some(self.standard_impact(envir.walking_cost(from, step.to).duration(self.speed())))
            }
        }
    }

    fn damage(
        &self,
        damage_writer: Either<
            &mut EventWriter<ActorEvent<Damage>>,
            &mut EventWriter<ItemEvent<Damage>>,
        >,
        envir: &Envir,
        infos: &Infos,
        hierarchy: &Hierarchy,
        damaged: Entity,
        damaged_pos: Pos,
        speed: MillimeterPerSecond,
    ) -> Impact {
        let mut melee_weapon = None;
        if let Some(body_containers) = self.body_containers {
            let (_, hands_children) = hierarchy.containers.get(body_containers.hands).unwrap();
            if let Some(hands_children) = hands_children {
                if let Some(&weapon) = hands_children.iter().next() {
                    let (_, definition, ..) = hierarchy.items.get(weapon).unwrap();
                    melee_weapon = infos.item(&definition.id);
                }
            }
        }

        let damage = Damage {
            attacker: self.name.single(),
            amount: self.melee.damage(melee_weapon),
        };
        match damage_writer {
            Either::Left(damage_writer) => damage_writer.send(ActorEvent::new(damaged, damage)),
            Either::Right(damage_writer) => damage_writer.send(ItemEvent {
                item_entity: damaged,
                change: damage,
            }),
        }

        // Needed when a character smashes something at it's own position
        let cost_pos = if *self.pos == damaged_pos {
            self.pos
                .offset(PosOffset {
                    x: 1,
                    level: LevelOffset::ZERO,
                    z: 0,
                })
                .unwrap()
        } else {
            damaged_pos
        };
        Impact::heavy(
            self.entity,
            envir.walking_cost(*self.pos, cost_pos).duration(speed),
        )
    }

    pub(crate) fn attack(
        &self,
        message_writer: &mut EventWriter<Message>,
        damage_writer: &mut EventWriter<ActorEvent<Damage>>,
        envir: &Envir,
        infos: &Infos,
        hierarchy: &Hierarchy,
        attack: &Attack,
    ) -> Option<Impact> {
        let Some(high_speed) = self.high_speed() else {
            message_writer.send(Message::warn(
                Phrase::from_name(self.name).add("is too exhausted to attack"),
            ));
            return None;
        };

        if !envir.are_nbors(*self.pos, attack.target) {
            unimplemented!();
        }

        if let Some((defender, _)) = envir.find_character(attack.target) {
            Some(self.damage(
                Either::Left(damage_writer),
                envir,
                infos,
                hierarchy,
                defender,
                attack.target,
                high_speed,
            ))
        } else {
            message_writer.send(Message::warn(
                Phrase::from_name(self.name).add("attacks nothing"),
            ));
            None
        }
    }

    pub(crate) fn smash(
        &self,
        message_writer: &mut EventWriter<Message>,
        damage_writer: &mut EventWriter<ItemEvent<Damage>>,
        envir: &Envir,
        infos: &Infos,
        hierarchy: &Hierarchy,
        smash: &Smash,
    ) -> Option<Impact> {
        let Some(high_speed) = self.high_speed() else {
            message_writer.send(Message::warn(
                Phrase::from_name(self.name).add("is too exhausted to smash"),
            ));
            return None;
        };

        if !envir.are_nbors(*self.pos, smash.target) && smash.target != *self.pos {
            unimplemented!();
        }

        let stair_pos = Pos::new(smash.target.x, self.pos.level, smash.target.z);
        if self.pos.level.up() == Some(smash.target.level)
            && envir.stairs_up_to(stair_pos).is_none()
        {
            message_writer.send(Message::warn(
                Phrase::from_name(self.name).add("smashes the ceiling"),
            ));
            None
        } else if self.pos.level.down() == Some(smash.target.level)
            && envir.stairs_down_to(stair_pos).is_none()
        {
            message_writer.send(Message::warn(
                Phrase::from_name(self.name).add("smashes the floor"),
            ));
            None
        } else if let Some((smashable, _)) = envir.find_smashable(smash.target) {
            Some(self.damage(
                Either::Right(damage_writer),
                envir,
                infos,
                hierarchy,
                smashable,
                smash.target,
                high_speed,
            ))
        } else {
            message_writer.send(Message::warn(
                Phrase::from_name(self.name).add("smashes nothing"),
            ));
            None
        }
    }

    pub(crate) fn close(
        &self,
        message_writer: &mut EventWriter<Message>,
        toggle_writer: &mut EventWriter<TerrainEvent<Toggle>>,
        envir: &Envir,
        close: &Close,
    ) -> Option<Impact> {
        if !envir.are_nbors(*self.pos, close.target) && close.target != *self.pos {
            unimplemented!();
        }

        if let Some((closeable, closeable_name)) = envir.find_closeable(close.target) {
            if let Some((_, character)) = envir.find_character(close.target) {
                message_writer.send(Message::warn(
                    Phrase::from_name(self.name)
                        .add("can't close")
                        .push(closeable_name.single())
                        .add("on")
                        .push(character.single()),
                ));
                None
            } else {
                toggle_writer.send(TerrainEvent {
                    terrain_entity: closeable,
                    change: Toggle::Close,
                });
                Some(
                    self.standard_impact(
                        envir
                            .walking_cost(*self.pos, close.target)
                            .duration(self.speed()),
                    ),
                )
            }
        } else {
            let missing = ObjectName::missing();
            let obstacle = envir.find_terrain(close.target).unwrap_or(&missing);
            message_writer.send(Message::warn(
                Phrase::from_name(self.name)
                    .add("can't close")
                    .push(obstacle.single()),
            ));
            None
        }
    }

    pub(crate) fn wield(
        &self,
        commands: &mut Commands,
        message_writer: &mut EventWriter<Message>,
        location: &mut Location,
        hierarchy: &Hierarchy,
        wield: &Wield,
    ) -> Option<Impact> {
        let impact = self.take(
            commands,
            message_writer,
            location,
            hierarchy,
            self.body_containers.unwrap().hands,
            wield.entity,
        );
        if impact.is_some() {
            commands.entity(wield.entity).insert(PlayerWielded);
        }
        impact
    }

    pub(crate) fn unwield(
        &self,
        commands: &mut Commands,
        message_writer: &mut EventWriter<Message>,
        location: &mut Location,
        hierarchy: &Hierarchy,
        unwield: &Unwield,
    ) -> Option<Impact> {
        let impact = self.take(
            commands,
            message_writer,
            location,
            hierarchy,
            self.body_containers.unwrap().clothing,
            unwield.entity,
        );
        if impact.is_some() {
            commands.entity(unwield.entity).remove::<PlayerWielded>();
        }
        impact
    }

    pub(crate) fn pickup(
        &self,
        commands: &mut Commands,
        message_writer: &mut EventWriter<Message>,
        location: &mut Location,
        hierarchy: &Hierarchy,
        pickup: &Pickup,
    ) -> Option<Impact> {
        self.take(
            commands,
            message_writer,
            location,
            hierarchy,
            self.body_containers.unwrap().clothing,
            pickup.entity,
        )
    }

    fn take(
        &self,
        commands: &mut Commands,
        message_writer: &mut EventWriter<Message>,
        location: &mut Location,
        hierarchy: &Hierarchy,
        container_entity: Entity,
        taken: Entity,
    ) -> Option<Impact> {
        if let Ok((
            taken_entity,
            taken_definition,
            taken_object_name,
            taken_pos,
            taken_amount,
            taken_filthy,
            taken_containable,
            taken_parent,
        )) = hierarchy.items.get(taken)
        {
            let taken_amount = taken_amount.unwrap_or(&Amount(1));
            let taken_parent = taken_parent.expect("Parent entity required");

            if let Some(taken_pos) = taken_pos {
                let offset = *taken_pos - *self.pos;
                assert!(
                    offset.x.abs() <= 1,
                    "Taking is not possible from more than one tile away"
                );
                assert!(
                    offset.level == LevelOffset::ZERO,
                    "Taking is only possible on the same level"
                );
                assert!(
                    offset.z.abs() <= 1,
                    "Taking is not possible from more than one tile away"
                );
            } else {
                assert!(
                    taken_parent.get() == self.body_containers.unwrap().hands
                        || taken_parent.get() == self.body_containers.unwrap().clothing,
                    "Item parents should be part of the body"
                );
            }

            let current_items = hierarchy
                .items
                .iter()
                .filter(|(.., parent)| parent.map(Parent::get) == Some(container_entity))
                .map(|(.., containable, _)| containable);

            let (container, _) = hierarchy.containers.get(container_entity).unwrap();
            match container.check_add(
                self.name.single(),
                current_items,
                taken_containable,
                taken_amount,
            ) {
                Ok(allowed_amount) => {
                    if &allowed_amount < taken_amount {
                        self.take_some(
                            commands,
                            message_writer,
                            location,
                            container_entity,
                            allowed_amount,
                            taken_entity,
                            taken_definition.clone(),
                            taken_object_name.clone(),
                            taken_pos,
                            taken_amount,
                            taken_filthy,
                            taken_containable.clone(),
                            taken_parent,
                        );
                    } else {
                        let taken_name =
                            taken_object_name.as_item(Some(taken_amount), taken_filthy);
                        self.take_all(
                            commands,
                            message_writer,
                            location,
                            container_entity,
                            taken_entity,
                            taken_name,
                        );
                    }
                    Some(self.activate())
                }
                Err(messages) => {
                    assert!(!messages.is_empty(), "Empty messages are not allowed");
                    message_writer.send_batch(messages);
                    None
                }
            }
        } else {
            message_writer.send(Message::warn(
                Phrase::new("Nothing to pick up for").push(self.name.single()),
            ));
            None
        }
    }

    fn take_some(
        &self,
        commands: &mut Commands,
        message_writer: &mut EventWriter<Message>,
        location: &mut Location,
        container_entity: Entity,
        allowed_amount: Amount,
        taken_entity: Entity,
        definition: ObjectDefinition,
        object_name: ObjectName,
        taken_pos: Option<&Pos>,
        split_amount: &Amount,
        filthy: Option<&Filthy>,
        containable: Containable,
        taken_parent: &Parent,
    ) {
        let taken_name = object_name.as_item(Some(&allowed_amount), filthy);
        message_writer.send(Message::info(
            Phrase::from_name(self.name)
                .add("picks up")
                .extend(taken_name),
        ));

        let left_over_amount = split_amount - &allowed_amount;
        //dbg!(&split_amount);
        //dbg!(&allowed_amount);
        //dbg!(&left_over_amount);
        let left_over_entity = commands
            .spawn((
                definition,
                object_name,
                left_over_amount,
                containable,
                LastSeen::Currently,
                SpatialBundle::default(),
            ))
            .set_parent(taken_parent.get())
            .id();
        if filthy.is_some() {
            commands.entity(left_over_entity).insert(Filthy);
        }
        if let Some(&taken_pos) = taken_pos {
            commands.entity(left_over_entity).insert(taken_pos);
        }
        location.update(left_over_entity, taken_pos.copied());

        commands
            .entity(taken_entity)
            .insert(allowed_amount)
            .remove::<Pos>()
            .set_parent(container_entity);
        location.update(taken_entity, None);
    }

    fn take_all(
        &self,
        commands: &mut Commands,
        message_writer: &mut EventWriter<Message>,
        location: &mut Location,
        container_entity: Entity,
        taken_entity: Entity,
        taken_name: Vec<Fragment>,
    ) {
        message_writer.send(Message::info(
            Phrase::from_name(self.name)
                .add("picks up")
                .extend(taken_name),
        ));
        commands
            .entity(container_entity)
            .push_children(&[taken_entity]);
        commands.entity(taken_entity).remove::<Pos>();
        location.update(taken_entity, None);
    }

    pub(crate) fn dump(
        &self,
        commands: &mut Commands,
        message_writer: &mut EventWriter<Message>,
        location: &mut Location,
        hierarchy: &Hierarchy,
        dump: &Dump,
    ) -> Option<Impact> {
        let (_, _, dumped_name, _, dumped_amount, dumped_filthy, _, dumped_parent) =
            hierarchy.items.get(dump.entity).unwrap();
        let dumped_parent = dumped_parent.map(Parent::get);

        if dumped_parent.is_none()
            || (dumped_parent != Some(self.body_containers.unwrap().hands)
                && dumped_parent != Some(self.body_containers.unwrap().clothing))
        {
            message_writer.send(Message::info(
                Phrase::from_name(self.name)
                    .add("can't drop")
                    .extend(dumped_name.as_item(dumped_amount, dumped_filthy))
                    .add(", because (s)he does not have it"),
            ));
            return None;
        }

        // TODO Check for obstacles
        let dumped_pos = self.pos.raw_nbor(Nbor::Horizontal(dump.direction)).unwrap();

        message_writer.send(Message::info(
            Phrase::from_name(self.name)
                .add("drops")
                .extend(dumped_name.as_item(dumped_amount, dumped_filthy)),
        ));
        commands
            .entity(dumped_parent.unwrap())
            .remove_children(&[dump.entity]);
        commands
            .entity(dump.entity)
            .insert(VisibilityBundle::default())
            .insert(dumped_pos);
        location.update(dump.entity, Some(*self.pos));
        Some(self.activate())
    }

    pub(crate) fn examine_item(
        message_writer: &mut EventWriter<Message>,
        infos: &Infos,
        definitions: &Query<&ObjectDefinition>,
        examine_item: &ExamineItem,
    ) {
        if let Ok(definition) = definitions.get(examine_item.entity) {
            if let Some(item_info) = infos.item(&definition.id) {
                if let Some(description) = &item_info.description {
                    message_writer.send(Message::info(Phrase::new(
                        match description {
                            Description::Simple(simple) => simple,
                            Description::Complex(complex) => complex.get("str").unwrap(),
                        }
                        .as_str(),
                    )));
                } else {
                    eprintln!("No description");
                }
            } else {
                eprintln!("No info");
            }
        } else {
            eprintln!("No definition");
        }
    }

    pub(crate) fn change_pace(&self, commands: &mut Commands) {
        commands
            .entity(self.entity)
            .insert(self.walking_mode.switch());
    }
}
