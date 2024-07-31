use crate::prelude::*;
use bevy::{ecs::query::QueryData, prelude::*};

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
            Subject::Other(Phrase::from_fragment(self.name.single(*self.pos)))
        }
    }

    pub(crate) fn speed(&self) -> Speed {
        self.base_speed
            .speed(self.walking_mode, self.stamina.breath())
    }

    fn high_speed(&self) -> Option<Speed> {
        match self.stamina.breath() {
            Breath::Normal | Breath::AlmostWinded => {
                Some(self.base_speed.speed(&WalkingMode::Running, Breath::Normal))
            }
            Breath::Winded => None,
        }
    }

    fn low_speed(&self) -> Option<Speed> {
        match self.stamina.breath() {
            Breath::Normal | Breath::AlmostWinded => {
                Some(self.base_speed.speed(&WalkingMode::Walking, Breath::Normal))
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

    const fn standard_impact(&self, timeout: Duration) -> Impact {
        Impact::new(
            self.entity,
            timeout,
            Some(self.walking_mode.stamina_impact(self.stamina.breath())),
        )
    }

    pub(crate) fn stay(&self) -> Impact {
        Impact::standing_rest(
            self.entity,
            Distance::ADJACENT / 2 / self.high_speed().unwrap_or_else(|| self.speed()),
        )
    }

    pub(crate) fn sleep(
        &mut self,
        healing_writer: &mut EventWriter<'_, ActorEvent<Healing>>,
        healing_durations: &mut Query<&mut HealingDuration>,
    ) -> Impact {
        let sleep_duration = Duration::MINUTE;

        let mut healing_duration = healing_durations
            .get_mut(self.entity)
            .expect("Actor entity should be found");

        let healing_amount = healing_duration.heal(sleep_duration);
        healing_writer.send(ActorEvent::new(
            self.entity,
            Healing {
                amount: healing_amount as u16,
            },
        ));

        Impact::laying_rest(self.entity, sleep_duration)
    }

    fn activate(&self) -> Impact {
        self.standard_impact(Distance::ADJACENT * 3 / self.speed())
    }

    pub(crate) fn step(
        &self,
        commands: &mut Commands,
        message_writer: &mut MessageWriter,
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
            //Collision::Fall(fall_pos) => {
            //    pos = fall_pos;
            //    location.add(mover, *pos);
            //    VERTICAL
            //}
            Collision::Blocked(obstacle) => {
                message_writer
                    .subject(self.subject())
                    .verb("crash", "es")
                    .add("into")
                    .push(obstacle.single(to))
                    .send_warn();
                None
            }
            Collision::Ledged => {
                message_writer
                    .subject(self.subject())
                    .verb("halt", "s")
                    .add("at the ledge")
                    .send_warn();
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

    fn damage<E: Event, N>(
        &self,
        damage_writer: &mut EventWriter<E>,
        envir: &Envir,
        infos: &Infos,
        hierarchy: &Hierarchy,
        damaged: Entity,
        damaged_pos: Pos,
        speed: Speed,
        new: N,
    ) -> Impact
    where
        N: Fn(Entity, Damage) -> E,
    {
        let mut melee_weapon = None;
        if let Some(body_containers) = self.body_containers {
            let mut hands_children = hierarchy.items_in(body_containers.hands);
            if let Some(weapon) = hands_children.next() {
                melee_weapon = infos.try_item(&weapon.definition.id);
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
        message_writer: &mut MessageWriter,
        damage_writer: &mut EventWriter<ActorEvent<Damage>>,
        envir: &Envir,
        infos: &Infos,
        hierarchy: &Hierarchy,
        attack: &Attack,
    ) -> Option<Impact> {
        let Some(high_speed) = self.high_speed() else {
            message_writer
                .subject(self.subject())
                .is()
                .add("too exhausted to attack")
                .send_error();
            return None;
        };

        let target = envir.get_nbor(*self.pos, attack.target).expect("Valid pos");

        if let Some((defender, _)) = envir.find_character(target) {
            Some(self.damage(
                damage_writer,
                envir,
                infos,
                hierarchy,
                defender,
                target,
                high_speed,
                ActorEvent::new,
            ))
        } else {
            message_writer
                .subject(self.subject())
                .verb("attack", "s")
                .add("nothing")
                .send_warn();
            None
        }
    }

    pub(crate) fn smash(
        &self,
        message_writer: &mut MessageWriter,
        damage_writer: &mut EventWriter<TerrainEvent<Damage>>,
        envir: &Envir,
        infos: &Infos,
        hierarchy: &Hierarchy,
        smash: &Smash,
    ) -> Option<Impact> {
        let Some(high_speed) = self.high_speed() else {
            message_writer
                .subject(self.subject())
                .is()
                .add("too exhausted to smash")
                .send_error();
            return None;
        };

        let target = envir.get_nbor(*self.pos, smash.target).expect("Valid pos");

        let stair_pos = Pos::new(target.x, self.pos.level, target.z);
        if self.pos.level.up() == Some(target.level) && envir.stairs_up_to(stair_pos).is_none() {
            message_writer
                .subject(self.subject())
                .verb("smash", "es")
                .add("the ceiling")
                .send_warn();
            None
        } else if self.pos.level.down() == Some(target.level)
            && envir.stairs_down_to(stair_pos).is_none()
        {
            message_writer
                .subject(self.subject())
                .verb("smash", "es")
                .add("the floor")
                .send_warn();
            None
        } else if let Some(smashable) = envir.find_smashable(target) {
            Some(self.damage(
                damage_writer,
                envir,
                infos,
                hierarchy,
                smashable,
                target,
                high_speed,
                TerrainEvent::new,
            ))
        } else {
            message_writer
                .subject(self.subject())
                .verb("smash", "es")
                .add("nothing")
                .send_warn();
            None
        }
    }

    pub(crate) fn pulp(
        &self,
        message_writer: &mut MessageWriter,
        corpse_damage_writer: &mut EventWriter<CorpseEvent<Damage>>,
        envir: &Envir,
        infos: &Infos,
        hierarchy: &Hierarchy,
        pulp: &Pulp,
    ) -> Option<Impact> {
        let Some(high_speed) = self.high_speed() else {
            message_writer
                .subject(self.subject())
                .is()
                .add("too exhausted to pulp")
                .send_warn();
            return None;
        };

        let target = self.pos.horizontal_nbor(pulp.target);

        if let Some(pulpable_entity) = envir.find_pulpable(target) {
            Some(self.damage(
                corpse_damage_writer,
                envir,
                infos,
                hierarchy,
                pulpable_entity,
                target,
                high_speed,
                CorpseEvent::new,
            ))
        } else {
            message_writer
                .subject(self.subject())
                .verb("pulp", "s")
                .add("nothing")
                .send_warn();
            None
        }
    }

    pub(crate) fn peek(
        &self,
        message_writer: &mut MessageWriter,
        player_action_state: &mut NextState<PlayerActionState>,
        envir: &Envir,
        peek: &Peek,
    ) -> Option<Impact> {
        let from = *self.pos;

        let to = envir
            .get_nbor(from, Nbor::Horizontal(peek.target.into()))
            .expect("Valid pos");

        match envir.collide(from, to, true) {
            Collision::Pass | Collision::Ledged => {
                if let Some(low_speed) = self.low_speed() {
                    player_action_state.set(PlayerActionState::Peeking {
                        direction: peek.target,
                    });
                    Some(self.standard_impact(envir.walking_cost(from, to).duration(low_speed)))
                } else {
                    message_writer
                        .subject(self.subject())
                        .is()
                        .add("too exhausted to peek")
                        .send_warn();
                    None
                }
            }
            _ => {
                message_writer.you("can't peek there").send_warn();
                None
            }
        }
    }

    pub(crate) fn close(
        &self,
        message_writer: &mut MessageWriter,
        toggle_writer: &mut EventWriter<TerrainEvent<Toggle>>,
        envir: &Envir,
        close: &Close,
    ) -> Option<Impact> {
        let target = self.pos.horizontal_nbor(close.target);

        if let Some((closeable, closeable_name)) = envir.find_closeable(target) {
            if let Some((_, character)) = envir.find_character(target) {
                message_writer
                    .subject(self.subject())
                    .simple("can't close")
                    .push(closeable_name.single(target))
                    .add("on")
                    .push(character.single(target))
                    .send_warn();
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
            message_writer
                .subject(self.subject())
                .simple("can't close")
                .push(obstacle.single(target))
                .send_warn();
            None
        }
    }

    pub(crate) fn wield(
        &self,
        commands: &mut Commands,
        message_writer: &mut MessageWriter,
        location: &mut Location,
        hierarchy: &Hierarchy,
        item: &ItemItem,
    ) -> Option<Impact> {
        let impact = self.take(
            commands,
            message_writer,
            location,
            &self.hands(hierarchy),
            item,
        );
        if impact.is_some() && self.player.is_some() {
            commands.entity(item.entity).insert(PlayerWielded);
        }
        impact
    }

    pub(crate) fn unwield(
        &self,
        commands: &mut Commands,
        message_writer: &mut MessageWriter,
        location: &mut Location,
        hierarchy: &Hierarchy,
        item: &ItemItem,
    ) -> Option<Impact> {
        let impact = self.take(
            commands,
            message_writer,
            location,
            &self.clothing(hierarchy),
            item,
        );
        if impact.is_some() {
            commands.entity(item.entity).remove::<PlayerWielded>();
        }
        impact
    }

    pub(crate) fn pickup(
        &self,
        commands: &mut Commands,
        message_writer: &mut MessageWriter,
        location: &mut Location,
        hierarchy: &Hierarchy,
        item: &ItemItem,
    ) -> Option<Impact> {
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
        message_writer: &mut MessageWriter,
        location: &mut Location,
        target: &Container,
        taken: &ItemItem,
    ) -> Option<Impact> {
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

        if let Ok(allowed_amount) = target.check_add(
            message_writer,
            self.subject(),
            taken.containable,
            taken.amount,
        ) {
            message_writer
                .subject(self.subject())
                .verb("pick", "s")
                .add("up")
                .extend(taken.fragments())
                .send_info();

            if &allowed_amount < taken.amount {
                Self::take_some(commands, location, target.entity, allowed_amount, taken);
            } else {
                Self::take_all(commands, location, target.entity, taken.entity);
            }
            Some(self.activate())
        } else {
            None
        }
    }

    fn take_some(
        commands: &mut Commands,
        location: &mut Location,
        container_entity: Entity,
        allowed_amount: Amount,
        taken: &ItemItem,
    ) {
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
        commands: &mut Commands,
        location: &mut Location,
        container_entity: Entity,
        taken_entity: Entity,
    ) {
        commands
            .entity(container_entity)
            .push_children(&[taken_entity]);
        commands.entity(taken_entity).remove::<Pos>();
        location.update(taken_entity, None);
    }

    pub(crate) fn move_item(
        &self,
        commands: &mut Commands,
        message_writer: &mut MessageWriter,
        subzone_level_entities: &SubzoneLevelEntities,
        location: &mut Location,
        moved: &ItemItem,
        to: Nbor,
    ) -> Option<Impact> {
        if let Some(from) = moved.pos {
            let offset = *from - *self.pos;
            let potentially_valid = HorizontalDirection::try_from(offset).is_ok()
                || matches!(offset.level, LevelOffset { h: -1 | 1 });
            if !potentially_valid {
                message_writer.str("Too far to move").send_error();
                return None;
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

        message_writer
            .subject(self.subject())
            .verb(if dump { "drop" } else { "move" }, "s")
            .extend(moved.fragments())
            .send_info();

        let Some(new_parent) = subzone_level_entities.get(SubzoneLevel::from(to)) else {
            message_writer
                .str("Subzone not found when moving an item")
                .send_error();
            return None;
        };
        commands
            .entity(moved.entity)
            .insert((VisibilityBundle::default(), to))
            .set_parent(new_parent);
        location.update(moved.entity, Some(*self.pos));
        Some(self.activate())
    }

    pub(crate) fn start_craft(
        &self,
        message_writer: &mut MessageWriter,
        next_player_action_state: &mut NextState<PlayerActionState>,
        spawner: &mut Spawner,
        infos: &Infos,
        subzone_level_entities: &SubzoneLevelEntities,
        start_craft: StartCraft,
    ) -> Option<Impact> {
        let pos = self.pos.horizontal_nbor(start_craft.target);
        let Some(parent_entity) = subzone_level_entities.get(SubzoneLevel::from(pos)) else {
            message_writer
                .str("Subzone not found when starting to craft")
                .send_error();
            return None;
        };

        // TODO consume components

        let item = spawner.spawn_craft(infos, parent_entity, pos, start_craft.recipe_id);

        next_player_action_state.set(PlayerActionState::Crafting { item });

        None
    }

    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn continue_craft(
        &self,
        commands: &mut Commands,
        message_writer: &mut MessageWriter,
        next_player_action_state: &mut NextState<PlayerActionState>,
        spawner: &mut Spawner,
        infos: &Infos,
        crafts: &mut Query<(Item, &mut Craft)>,
        craft: Entity,
    ) -> Option<Impact> {
        let (item, mut craft) = crafts.get_mut(craft).expect("Craft should be found");

        let crafting_progress = Duration::SECOND * 3;

        craft.work(crafting_progress);
        if craft.finished() {
            message_writer.you("finish").add("crafting").send_info();
            let parent = item.parent.get();
            let pos = *item.pos.unwrap_or(self.pos);
            let amount = *item.amount;
            let cdda_item = CddaItem::from(craft.object_id.clone());
            commands.entity(item.entity).despawn_recursive();
            _ = spawner.spawn_item(infos, parent, pos, &cdda_item, amount);
            next_player_action_state.set(PlayerActionState::Normal);
        }

        Some(Impact {
            actor_entity: self.entity,
            timeout: crafting_progress,
            stamina_impact: Some(StaminaImpact::Neutral),
        })
    }

    pub(crate) fn examine_item(message_writer: &mut MessageWriter, infos: &Infos, item: &ItemItem) {
        if let Some(item_info) = infos.try_item(&item.definition.id) {
            if let Some(description) = &item_info.description {
                message_writer
                    .str(match description {
                        Description::Simple(simple) => simple,
                        Description::Complex(complex) => complex.get("str").expect("'str' key"),
                    })
                    .send_info();
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
