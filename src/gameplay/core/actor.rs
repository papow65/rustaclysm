use crate::prelude::*;
use bevy::{ecs::query::QueryData, prelude::*};

/** Derived from stamina */
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Breath {
    Normal,
    AlmostWinded,
    Winded,
}

#[must_use]
#[derive(Debug)]
pub(crate) struct Impact {
    pub(crate) actor_entity: Entity,
    pub(crate) timeout: Milliseconds,
    pub(crate) stamina_impact: Option<StaminaImpact>,
}

impl Impact {
    const fn new(
        actor_entity: Entity,
        timeout: Milliseconds,
        stamina_impact: Option<StaminaImpact>,
    ) -> Self {
        Self {
            actor_entity,
            timeout,
            stamina_impact,
        }
    }

    const fn none(actor_entity: Entity) -> Self {
        Self::new(actor_entity, Milliseconds::ZERO, None)
    }

    const fn rest(actor_entity: Entity, timeout: Milliseconds) -> Self {
        Self::new(actor_entity, timeout, Some(StaminaImpact::Rest))
    }

    const fn full_rest(actor_entity: Entity, timeout: Milliseconds) -> Self {
        Self::new(actor_entity, timeout, Some(StaminaImpact::FullRest))
    }

    const fn heavy(actor_entity: Entity, timeout: Milliseconds) -> Self {
        Self::new(actor_entity, timeout, Some(StaminaImpact::Heavy))
    }

    pub(crate) fn check_validity(&self) {
        assert_eq!(
            self.timeout == Milliseconds::ZERO,
            self.stamina_impact.is_none(),
            "{self:?} is invalid"
        );
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

#[derive(QueryData)]
#[query_data(derive(Debug))]
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

    #[allow(unused)]
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
            Breath::Normal | Breath::AlmostWinded => {
                Some(self.base_speed.speed(&WalkingMode::Running, Breath::Normal))
            }
            Breath::Winded => None,
        }
    }

    fn hands<'a>(&self, hierarchy: &'a Hierarchy) -> Container<'a> {
        Container::new(
            self.body_containers.expect("Body containers present").hands,
            hierarchy,
        )
    }

    fn clothing<'a>(&self, hierarchy: &'a Hierarchy) -> Container<'a> {
        Container::new(
            self.body_containers
                .expect("Body containers present")
                .clothing,
            hierarchy,
        )
    }

    const fn no_impact(&self) -> Impact {
        Impact::none(self.entity)
    }

    const fn standard_impact(&self, timeout: Milliseconds) -> Impact {
        Impact::new(
            self.entity,
            timeout,
            Some(self.walking_mode.stamina_impact(self.stamina.breath())),
        )
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
    ) -> Impact {
        let from = *self.pos;
        let to = envir.get_nbor(from, step.to).expect("Valid pos");

        match envir.collide(from, to, true) {
            Collision::Pass => {
                commands.entity(self.entity).insert(to);
                envir.location.update(self.entity, Some(to));
                self.standard_impact(envir.walking_cost(from, to).duration(self.speed()))
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
                self.no_impact()
            }
            Collision::Ledged => {
                message_writer.send(Message::warn(
                    self.subject().verb("halt", "s").add("at the ledge"),
                ));
                self.no_impact()
            }
            Collision::Opened(door) => {
                toggle_writer.send(TerrainEvent {
                    terrain_entity: door,
                    change: Toggle::Open,
                });
                self.standard_impact(envir.walking_cost(from, to).duration(self.speed()))
            }
        }
    }

    fn damage<E: Event, N>(
        &self,
        damage_writer: &mut EventWriter<E>,
        envir: &Envir,
        infos: &Infos,
        hierarchy: &Hierarchy,
        damaged: Entity,
        damaged_pos: Pos,
        speed: MillimeterPerSecond,
        new: N,
    ) -> Impact
    where
        N: Fn(Entity, Damage) -> E,
    {
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
        damage_writer.send(new(damaged, damage));

        // Needed when a character smashes something at it's own position
        let cost_pos = if *self.pos == damaged_pos {
            self.pos.horizontal_offset(1, 0)
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
    ) -> Impact {
        let Some(high_speed) = self.high_speed() else {
            message_writer.send(Message::warn(
                self.subject().is().add("too exhausted to attack"),
            ));
            return self.no_impact();
        };

        let target = envir.get_nbor(*self.pos, attack.target).expect("Valid pos");

        if let Some((defender, _)) = envir.find_character(target) {
            self.damage(
                damage_writer,
                envir,
                infos,
                hierarchy,
                defender,
                target,
                high_speed,
                ActorEvent::new,
            )
        } else {
            message_writer.send(Message::warn(
                self.subject().verb("attack", "s").add("nothing"),
            ));
            self.no_impact()
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
    ) -> Impact {
        let Some(high_speed) = self.high_speed() else {
            message_writer.send(Message::warn(
                self.subject().is().add("too exhausted to smash"),
            ));
            return self.no_impact();
        };

        let target = envir.get_nbor(*self.pos, smash.target).expect("Valid pos");

        let stair_pos = Pos::new(target.x, self.pos.level, target.z);
        if self.pos.level.up() == Some(target.level) && envir.stairs_up_to(stair_pos).is_none() {
            message_writer.send(Message::warn(
                self.subject().verb("smash", "es").add("the ceiling"),
            ));
            self.no_impact()
        } else if self.pos.level.down() == Some(target.level)
            && envir.stairs_down_to(stair_pos).is_none()
        {
            message_writer.send(Message::warn(
                self.subject().verb("smash", "es").add("the floor"),
            ));
            self.no_impact()
        } else if let Some(smashable) = envir.find_smashable(target) {
            self.damage(
                damage_writer,
                envir,
                infos,
                hierarchy,
                smashable,
                target,
                high_speed,
                TerrainEvent::new,
            )
        } else {
            message_writer.send(Message::warn(
                self.subject().verb("smash", "es").add("nothing"),
            ));
            self.no_impact()
        }
    }

    pub(crate) fn pulp(
        &self,
        message_writer: &mut EventWriter<Message>,
        corpse_damage_writer: &mut EventWriter<CorpseEvent<Damage>>,
        envir: &Envir,
        infos: &Infos,
        hierarchy: &Hierarchy,
        pulp: &Pulp,
    ) -> Impact {
        let Some(high_speed) = self.high_speed() else {
            message_writer.send(Message::warn(
                self.subject().is().add("too exhausted to pulp"),
            ));
            return self.no_impact();
        };

        let target = self.pos.horizontal_nbor(pulp.target);

        if let Some(pulpable_entity) = envir.find_pulpable(target) {
            self.damage(
                corpse_damage_writer,
                envir,
                infos,
                hierarchy,
                pulpable_entity,
                target,
                high_speed,
                CorpseEvent::new,
            )
        } else {
            message_writer.send(Message::warn(
                self.subject().verb("pulp", "s").add("nothing"),
            ));
            self.no_impact()
        }
    }

    pub(crate) fn close(
        &self,
        message_writer: &mut EventWriter<Message>,
        toggle_writer: &mut EventWriter<TerrainEvent<Toggle>>,
        envir: &Envir,
        close: &Close,
    ) -> Impact {
        let target = self.pos.horizontal_nbor(close.target);

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
                self.no_impact()
            } else {
                toggle_writer.send(TerrainEvent {
                    terrain_entity: closeable,
                    change: Toggle::Close,
                });
                self.standard_impact(envir.walking_cost(*self.pos, target).duration(self.speed()))
            }
        } else {
            let missing = ObjectName::missing();
            let obstacle = envir.find_terrain(target).unwrap_or(&missing);
            message_writer.send(Message::warn(
                Phrase::from_name(self.name)
                    .add("can't close")
                    .push(obstacle.single()),
            ));
            self.no_impact()
        }
    }

    pub(crate) fn wield(
        &self,
        commands: &mut Commands,
        message_writer: &mut EventWriter<Message>,
        location: &mut Location,
        hierarchy: &Hierarchy,
        item: &ItemItem,
    ) -> Impact {
        let impact = self.take(
            commands,
            message_writer,
            location,
            &self.hands(hierarchy),
            item,
        );
        if impact.stamina_impact.is_some() && self.player.is_some() {
            commands.entity(item.entity).insert(PlayerWielded);
        }
        impact
    }

    pub(crate) fn unwield(
        &self,
        commands: &mut Commands,
        message_writer: &mut EventWriter<Message>,
        location: &mut Location,
        hierarchy: &Hierarchy,
        item: &ItemItem,
    ) -> Impact {
        let impact = self.take(
            commands,
            message_writer,
            location,
            &self.clothing(hierarchy),
            item,
        );
        if impact.stamina_impact.is_some() {
            commands.entity(item.entity).remove::<PlayerWielded>();
        }
        impact
    }

    pub(crate) fn pickup(
        &self,
        commands: &mut Commands,
        message_writer: &mut EventWriter<Message>,
        location: &mut Location,
        hierarchy: &Hierarchy,
        item: &ItemItem,
    ) -> Impact {
        self.take(
            commands,
            message_writer,
            location,
            &self.clothing(hierarchy),
            item,
        )
    }

    fn take(
        &self,
        commands: &mut Commands,
        message_writer: &mut EventWriter<Message>,
        location: &mut Location,
        target: &Container,
        taken: &ItemItem,
    ) -> Impact {
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
                taken.parent.get() == self.body_containers.expect("Body containers present").hands
                    || taken.parent.get()
                        == self
                            .body_containers
                            .expect("Body containers present")
                            .clothing,
                "Item parents should be part of the body"
            );
        }

        match target.check_add(self.name.single(), taken.containable, taken.amount) {
            Ok(allowed_amount) => {
                if &allowed_amount < taken.amount {
                    self.take_some(
                        commands,
                        message_writer,
                        location,
                        target.entity,
                        allowed_amount,
                        taken,
                    );
                } else {
                    self.take_all(
                        commands,
                        message_writer,
                        location,
                        target.entity,
                        taken.entity,
                        taken.fragments(),
                    );
                }
                self.activate()
            }
            Err(messages) => {
                assert!(!messages.is_empty(), "Empty messages are not allowed");
                message_writer.send_batch(messages);
                self.no_impact()
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
            .set_parent(taken.parent.get())
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
        subzone_level_entities: &SubzoneLevelEntities,
        location: &mut Location,
        moved: &ItemItem,
        to: Nbor,
    ) -> Impact {
        if let Some(from) = moved.pos {
            let offset = *from - *self.pos;
            let potentially_valid = HorizontalDirection::try_from(offset).is_ok()
                || matches!(offset.level, LevelOffset { h: -1 | 1 });
            if !potentially_valid {
                message_writer.send(Message::error(Phrase::new("Too far to move")));
                return self.no_impact();
            }
        }
        let dump = moved.parent.get()
            == self.body_containers.expect("Body containers present").hands
            || moved.parent.get()
                == self
                    .body_containers
                    .expect("Body containers present")
                    .clothing;

        // TODO Check for obstacles
        let to = self.pos.raw_nbor(to).expect("Valid position");

        message_writer.send(Message::info(
            self.subject()
                .verb(if dump { "drop" } else { "move" }, "s")
                .extend(moved.fragments()),
        ));

        let Some(new_parent) = subzone_level_entities.get(SubzoneLevel::from(to)) else {
            eprintln!("Subzone for moving not found");
            return self.no_impact();
        };
        commands
            .entity(moved.entity)
            .insert((VisibilityBundle::default(), to))
            .set_parent(new_parent);
        location.update(moved.entity, Some(*self.pos));
        self.activate()
    }

    pub(crate) fn examine_item(
        message_writer: &mut EventWriter<Message>,
        infos: &Infos,
        item: &ItemItem,
    ) {
        if let Some(item_info) = infos.item(&item.definition.id) {
            if let Some(description) = &item_info.description {
                message_writer.send(Message::info(Phrase::new(
                    match description {
                        Description::Simple(simple) => simple,
                        Description::Complex(complex) => complex.get("str").expect("'str' key"),
                    }
                    .as_str(),
                )));
            } else {
                eprintln!("No description");
            }
        } else {
            eprintln!("No info");
        }
    }

    pub(crate) fn change_pace(&self, commands: &mut Commands) {
        commands
            .entity(self.entity)
            .insert(self.walking_mode.switch());
    }
}
