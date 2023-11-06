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

#[derive(Clone, Debug)]
pub(crate) enum Subject {
    You,
    Other(Phrase),
}

impl Subject {
    fn phrase(self, second_person: &str, third_person: String) -> Phrase {
        match self {
            Self::You => Phrase::from_fragment(Fragment {
                text: String::from("You"),
                color: Some(GOOD_TEXT_COLOR),
            })
            .add(second_person),
            Self::Other(phrase) => phrase.add(third_person),
        }
    }

    pub(crate) fn verb(self, root: &str, suffix: &str) -> Phrase {
        self.phrase(root, String::from(root) + suffix)
    }

    pub(crate) fn is(self) -> Phrase {
        self.phrase("is", String::from("are"))
    }
}

#[derive(WorldQuery)]
#[world_query(derive(Debug))]
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
    pub(crate) player: Option<&'static Player>,
}

impl ActorItem<'_> {
    pub(crate) fn subject(&self) -> Subject {
        if self.player.is_some() {
            Subject::You
        } else {
            Subject::Other(Phrase::from_name(self.name))
        }
    }

    pub(crate) fn fragment(&self) -> Fragment {
        if self.player.is_some() {
            Fragment {
                text: String::from("you"),
                color: Some(GOOD_TEXT_COLOR),
            }
        } else {
            self.name.single()
        }
    }

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
        let to = envir.get_nbor(from, step.to).expect("Valid pos");

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
                message_writer.send(Message::warn(
                    self.subject()
                        .verb("crash", "es")
                        .add("into")
                        .push(obstacle.single()),
                ));
                None
            }
            Collision::Ledged => {
                message_writer.send(Message::warn(
                    self.subject().verb("halt", "s").add("at the ledge"),
                ));
                None
            }
            Collision::Opened(door) => {
                toggle_writer.send(TerrainEvent {
                    terrain_entity: door,
                    change: Toggle::Open,
                });
                Some(self.standard_impact(envir.walking_cost(from, to).duration(self.speed())))
            }
        }
    }

    fn damage(
        &self,
        damage_writer: Either<
            &mut EventWriter<ActorEvent<Damage>>,
            &mut EventWriter<TerrainEvent<Damage>>,
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
            let mut hands_children = hierarchy.items_in(body_containers.hands);
            if let Some(weapon) = hands_children.next() {
                melee_weapon = infos.item(&weapon.definition.id);
            }
        }

        let damage = Damage {
            attacker: self.subject(),
            amount: self.melee.damage(melee_weapon),
        };
        match damage_writer {
            Either::Left(damage_writer) => damage_writer.send(ActorEvent::new(damaged, damage)),
            Either::Right(damage_writer) => damage_writer.send(TerrainEvent {
                terrain_entity: damaged,
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
                self.subject().is().add("too exhausted to attack"),
            ));
            return None;
        };

        let target = envir.get_nbor(*self.pos, attack.target).expect("Valid pos");

        if let Some((defender, _)) = envir.find_character(target) {
            Some(self.damage(
                Either::Left(damage_writer),
                envir,
                infos,
                hierarchy,
                defender,
                target,
                high_speed,
            ))
        } else {
            message_writer.send(Message::warn(
                self.subject().verb("attack", "s").add("nothing"),
            ));
            None
        }
    }

    pub(crate) fn smash(
        &self,
        message_writer: &mut EventWriter<Message>,
        damage_writer: &mut EventWriter<TerrainEvent<Damage>>,
        envir: &Envir,
        infos: &Infos,
        hierarchy: &Hierarchy,
        smash: &Smash,
    ) -> Option<Impact> {
        let Some(high_speed) = self.high_speed() else {
            message_writer.send(Message::warn(
                self.subject().is().add("too exhausted to smash"),
            ));
            return None;
        };

        let target = envir.get_nbor(*self.pos, smash.target).expect("Valid pos");

        let stair_pos = Pos::new(target.x, self.pos.level, target.z);
        if self.pos.level.up() == Some(target.level) && envir.stairs_up_to(stair_pos).is_none() {
            message_writer.send(Message::warn(
                self.subject().verb("smash", "es").add("the ceiling"),
            ));
            None
        } else if self.pos.level.down() == Some(target.level)
            && envir.stairs_down_to(stair_pos).is_none()
        {
            message_writer.send(Message::warn(
                self.subject().verb("smash", "es").add("the floor"),
            ));
            None
        } else if let Some((smashable, _)) = envir.find_smashable(target) {
            Some(self.damage(
                Either::Right(damage_writer),
                envir,
                infos,
                hierarchy,
                smashable,
                target,
                high_speed,
            ))
        } else {
            message_writer.send(Message::warn(
                self.subject().verb("smash", "es").add("nothing"),
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
        let target = envir.get_nbor(*self.pos, close.target).expect("Valid pos");

        if let Some((closeable, closeable_name)) = envir.find_closeable(target) {
            if let Some((_, character)) = envir.find_character(target) {
                message_writer.send(Message::warn(
                    self.subject()
                        .verb("can't", "")
                        .add("close")
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
                        envir.walking_cost(*self.pos, target).duration(self.speed()),
                    ),
                )
            }
        } else {
            let missing = ObjectName::missing();
            let obstacle = envir.find_terrain(target).unwrap_or(&missing);
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
            wield.item,
        );
        if impact.is_some() {
            commands.entity(wield.item).insert(PlayerWielded);
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
            unwield.item,
        );
        if impact.is_some() {
            commands.entity(unwield.item).remove::<PlayerWielded>();
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
            pickup.item,
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
        let taken = hierarchy.get_item(taken);

        let taken_parent = taken.parent.expect("Parent entity required");

        if let Some(taken_pos) = taken.pos {
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
            .items_in(container_entity)
            .map(|item| item.containable);
        let container = hierarchy.get_container(container_entity);
        match container.check_add(
            self.name.single(),
            current_items,
            taken.containable,
            taken.amount,
        ) {
            Ok(allowed_amount) => {
                if &allowed_amount < taken.amount {
                    self.take_some(
                        commands,
                        message_writer,
                        location,
                        container_entity,
                        allowed_amount,
                        &taken,
                        taken_parent,
                    );
                } else {
                    self.take_all(
                        commands,
                        message_writer,
                        location,
                        container_entity,
                        taken.entity,
                        taken.fragments(),
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
    }

    fn take_some(
        &self,
        commands: &mut Commands,
        message_writer: &mut EventWriter<Message>,
        location: &mut Location,
        container_entity: Entity,
        allowed_amount: Amount,
        taken: &ItemItem,
        taken_parent: &Parent,
    ) {
        message_writer.send(Message::info(
            self.subject()
                .verb("pick", "s")
                .add("up")
                .extend(taken.fragments()),
        ));

        let left_over_amount = taken.amount - &allowed_amount;
        //dbg!(&split_amount);
        //dbg!(&allowed_amount);
        //dbg!(&left_over_amount);
        let left_over_entity = commands
            .spawn((
                taken.definition.clone(),
                taken.name.clone(),
                left_over_amount,
                taken.containable.clone(),
                LastSeen::Currently,
                SpatialBundle::default(),
            ))
            .set_parent(taken_parent.get())
            .id();
        if taken.filthy.is_some() {
            commands.entity(left_over_entity).insert(Filthy);
        }
        if let Some(&taken_pos) = taken.pos {
            commands.entity(left_over_entity).insert(taken_pos);
        }
        location.update(left_over_entity, taken.pos.copied());

        commands
            .entity(taken.entity)
            .insert(allowed_amount)
            .remove::<Pos>()
            .set_parent(container_entity);
        location.update(taken.entity, None);
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
            self.subject()
                .verb("pick", "s")
                .add("up")
                .extend(taken_name),
        ));
        commands
            .entity(container_entity)
            .push_children(&[taken_entity]);
        commands.entity(taken_entity).remove::<Pos>();
        location.update(taken_entity, None);
    }

    pub(crate) fn move_item(
        &self,
        commands: &mut Commands,
        message_writer: &mut EventWriter<Message>,
        location: &mut Location,
        hierarchy: &Hierarchy,
        move_item: &MoveItem,
    ) -> Option<Impact> {
        let moved = hierarchy.get_item(move_item.item);

        if let Some(from) = moved.pos {
            let offset = *from - *self.pos;
            let potentially_valid = HorizontalDirection::try_from(offset).is_ok()
                || matches!(offset.level, LevelOffset { h: -1 | 1 });
            if !potentially_valid {
                message_writer.send(Message::error(Phrase::new("Too far to move")));
                return None;
            }
        }
        let moved_parent = moved.parent.map(Parent::get);
        let dump = moved_parent == Some(self.body_containers.unwrap().hands)
            || moved_parent == Some(self.body_containers.unwrap().clothing);

        // TODO Check for obstacles
        let to = self.pos.raw_nbor(move_item.to).unwrap();

        message_writer.send(Message::info(
            self.subject()
                .verb(if dump { "drop" } else { "move" }, "s")
                .extend(moved.fragments()),
        ));
        if dump {
            commands
                .entity(moved_parent.unwrap())
                .remove_children(&[move_item.item]);
        }
        commands
            .entity(move_item.item)
            .insert(VisibilityBundle::default())
            .insert(to);
        location.update(move_item.item, Some(*self.pos));
        Some(self.activate())
    }

    pub(crate) fn examine_item(
        message_writer: &mut EventWriter<Message>,
        infos: &Infos,
        definitions: &Query<&ObjectDefinition>,
        examine_item: &ExamineItem,
    ) {
        if let Ok(definition) = definitions.get(examine_item.item) {
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
