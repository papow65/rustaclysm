use crate::{gameplay::*, hud::text_color_expect_full};
use bevy::ecs::query::QueryData;
use bevy::prelude::{
    BuildChildren, Commands, DespawnRecursiveExt, Entity, Event, EventWriter, NextState, Query,
    Transform, Visibility,
};
use cdda_json_files::{CddaItem, Description};
use units::{Distance, Duration, Speed};

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

    const fn no_impact(&self) -> ActorImpact {
        ActorImpact::none(self.entity)
    }

    pub(crate) fn impact_from_duration(
        &self,
        duration: Duration,
        cost_per_second: StaminaCost,
    ) -> ActorImpact {
        ActorImpact::by_duration(self.entity, duration, cost_per_second)
    }

    pub(crate) fn impact_from_nbor(
        &self,
        duration: Duration,
        cost_per_meter: StaminaCost,
        nbor: Nbor,
    ) -> ActorImpact {
        ActorImpact::by_nbor(self.entity, duration, cost_per_meter, nbor)
    }

    pub(crate) fn stay(&self) -> ActorImpact {
        self.impact_from_duration(
            Distance::ADJACENT / 2 / self.high_speed().unwrap_or_else(|| self.speed()),
            StaminaCost::STANDING_REST,
        )
    }

    pub(crate) fn sleep(
        &self,
        message_writer: &mut MessageWriter,
        healing_writer: &mut EventWriter<'_, ActorEvent<Healing>>,
        player_action_state: &PlayerActionState,
        clock: &Clock,
        healing_durations: &mut Query<&mut HealingDuration>,
    ) -> ActorImpact {
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

        if let PlayerActionState::Sleeping { from } = player_action_state {
            let total_duration = clock.time() - *from;
            let color = text_color_expect_full(total_duration / (Duration::HOUR * 8));
            message_writer
                .you("sleep for")
                .push(Fragment::colorized(total_duration.short_format(), color))
                .send(Severity::Info, true);
        } else {
            eprintln!("Unexpected {player_action_state:?} while sleeping");
        }

        self.impact_from_duration(sleep_duration, StaminaCost::LYING_REST)
    }

    pub(crate) fn step(
        &self,
        commands: &mut Commands,
        message_writer: &mut MessageWriter,
        toggle_writer: &mut EventWriter<TerrainEvent<Toggle>>,
        envir: &mut Envir,
        step: &Step,
    ) -> ActorImpact {
        let from = *self.pos;
        let to = envir.get_nbor(from, step.to).expect("Valid pos");

        match envir.collide(from, to, true) {
            Collision::Pass => {
                commands.entity(self.entity).insert(to);
                envir.location.move_(self.entity, to);
                let walking_cost = envir.walking_cost(from, to);
                self.impact_from_nbor(
                    walking_cost.duration(self.speed()),
                    self.walking_mode.stamina_impact(self.stamina.breath()),
                    step.to,
                )
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
                self.no_impact()
            }
            Collision::Ledged => {
                message_writer
                    .subject(self.subject())
                    .verb("halt", "s")
                    .add("at the ledge")
                    .send_warn();
                self.no_impact()
            }
            Collision::Opened(door) => {
                toggle_writer.send(TerrainEvent {
                    terrain_entity: door,
                    change: Toggle::Open,
                });
                self.impact_from_duration(Duration::SECOND, StaminaCost::NEUTRAL)
            }
        }
    }

    fn damage<E: Event, N>(
        &self,
        damage_writer: &mut EventWriter<E>,
        infos: &Infos,
        hierarchy: &Hierarchy,
        damaged: Entity,
        new: N,
    ) -> ActorImpact
    where
        N: Fn(Entity, Damage) -> E,
    {
        let mut melee_weapon = None;
        if let Some(body_containers) = self.body_containers {
            let mut hands_children = hierarchy.items_in(body_containers.hands);
            if let Some(weapon) = hands_children.next() {
                melee_weapon = infos.try_common_item_info(&weapon.definition.id);
            }
        }

        let damage = Damage {
            attacker: self.subject(),
            amount: self.melee.damage(melee_weapon),
        };
        damage_writer.send(new(damaged, damage));

        self.impact_from_duration(Duration::SECOND, StaminaCost::HEAVY)
    }

    pub(crate) fn attack(
        &self,
        message_writer: &mut MessageWriter,
        damage_writer: &mut EventWriter<ActorEvent<Damage>>,
        envir: &Envir,
        infos: &Infos,
        hierarchy: &Hierarchy,
        attack: &Attack,
    ) -> ActorImpact {
        if self.stamina.breath() == Breath::Winded {
            message_writer
                .subject(self.subject())
                .is()
                .add("too exhausted to attack")
                .send_error();
            return self.no_impact();
        };

        let target = envir.get_nbor(*self.pos, attack.target).expect("Valid pos");

        if let Some((defender, _)) = envir.find_character(target) {
            self.damage(damage_writer, infos, hierarchy, defender, ActorEvent::new)
        } else {
            message_writer
                .subject(self.subject())
                .verb("attack", "s")
                .add("nothing")
                .send_warn();
            self.no_impact()
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
    ) -> ActorImpact {
        if self.stamina.breath() == Breath::Winded {
            message_writer
                .subject(self.subject())
                .is()
                .add("too exhausted to smash")
                .send_error();
            return self.no_impact();
        };

        let target = envir.get_nbor(*self.pos, smash.target).expect("Valid pos");

        let stair_pos = Pos::new(target.x, self.pos.level, target.z);
        if self.pos.level.up() == Some(target.level) && envir.stairs_up_to(stair_pos).is_none() {
            message_writer
                .subject(self.subject())
                .verb("smash", "es")
                .add("the ceiling")
                .send_warn();
            self.no_impact()
        } else if self.pos.level.down() == Some(target.level)
            && envir.stairs_down_to(stair_pos).is_none()
        {
            message_writer
                .subject(self.subject())
                .verb("smash", "es")
                .add("the floor")
                .send_warn();
            self.no_impact()
        } else if let Some(smashable) = envir.find_smashable(target) {
            self.damage(
                damage_writer,
                infos,
                hierarchy,
                smashable,
                TerrainEvent::new,
            )
        } else {
            message_writer
                .subject(self.subject())
                .verb("smash", "es")
                .add("nothing")
                .send_warn();
            self.no_impact()
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
    ) -> ActorImpact {
        if self.stamina.breath() == Breath::Winded {
            message_writer
                .subject(self.subject())
                .is()
                .add("too exhausted to pulp")
                .send_warn();
            return self.no_impact();
        };

        let target = self.pos.horizontal_nbor(pulp.target);

        if let Some(pulpable_entity) = envir.find_pulpable(target) {
            self.damage(
                corpse_damage_writer,
                infos,
                hierarchy,
                pulpable_entity,
                CorpseEvent::new,
            )
        } else {
            message_writer
                .subject(self.subject())
                .verb("pulp", "s")
                .add("nothing")
                .send_warn();
            self.no_impact()
        }
    }

    pub(crate) fn peek(
        &self,
        message_writer: &mut MessageWriter,
        player_action_state: &mut NextState<PlayerActionState>,
        envir: &Envir,
        peek: &Peek,
    ) -> ActorImpact {
        let from = *self.pos;

        let to = envir
            .get_nbor(from, Nbor::Horizontal(peek.target.into()))
            .expect("Valid pos");

        match envir.collide(from, to, true) {
            Collision::Pass | Collision::Ledged => match self.stamina.breath() {
                Breath::Normal | Breath::AlmostWinded => {
                    player_action_state.set(PlayerActionState::Peeking {
                        direction: peek.target,
                    });
                    self.impact_from_duration(Duration::SECOND, StaminaCost::NEUTRAL)
                }
                Breath::Winded => {
                    message_writer
                        .subject(self.subject())
                        .is()
                        .add("too exhausted to peek")
                        .send_warn();
                    self.no_impact()
                }
            },
            _ => {
                message_writer.you("can't peek there").send_warn();
                self.no_impact()
            }
        }
    }

    pub(crate) fn close(
        &self,
        message_writer: &mut MessageWriter,
        toggle_writer: &mut EventWriter<TerrainEvent<Toggle>>,
        envir: &Envir,
        close: &Close,
    ) -> ActorImpact {
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
                self.no_impact()
            } else {
                toggle_writer.send(TerrainEvent {
                    terrain_entity: closeable,
                    change: Toggle::Close,
                });
                self.impact_from_duration(Duration::SECOND, StaminaCost::NEUTRAL)
            }
        } else {
            let missing = ObjectName::missing();
            let obstacle = envir.find_terrain(target).unwrap_or(&missing);
            message_writer
                .subject(self.subject())
                .simple("can't close")
                .push(obstacle.single(target))
                .send_warn();
            self.no_impact()
        }
    }

    pub(crate) fn wield(
        &self,
        commands: &mut Commands,
        message_writer: &mut MessageWriter,
        hierarchy: &Hierarchy,
        item: &ItemItem,
    ) -> ActorImpact {
        let impact = self.take(commands, message_writer, &self.hands(hierarchy), item);
        if impact.is_some() && self.player.is_some() {
            commands.entity(item.entity).insert(PlayerWielded);
        }
        impact
    }

    pub(crate) fn unwield(
        &self,
        commands: &mut Commands,
        message_writer: &mut MessageWriter,
        hierarchy: &Hierarchy,
        item: &ItemItem,
    ) -> ActorImpact {
        let impact = self.take(commands, message_writer, &self.clothing(hierarchy), item);
        if impact.is_some() {
            commands.entity(item.entity).remove::<PlayerWielded>();
        }
        impact
    }

    pub(crate) fn pickup(
        &self,
        commands: &mut Commands,
        message_writer: &mut MessageWriter,
        hierarchy: &Hierarchy,
        item: &ItemItem,
    ) -> ActorImpact {
        self.take(commands, message_writer, &self.clothing(hierarchy), item)
    }

    fn take(
        &self,
        commands: &mut Commands,
        message_writer: &mut MessageWriter,
        target: &Container,
        taken: &ItemItem,
    ) -> ActorImpact {
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
                Self::take_some(commands, target.entity, allowed_amount, taken);
            } else {
                Self::take_all(commands, target.entity, taken.entity);
            }
            self.impact_from_duration(Duration::SECOND, StaminaCost::NEUTRAL)
        } else {
            self.no_impact()
        }
    }

    fn take_some(
        commands: &mut Commands,
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
                Transform::default(),
                Visibility::default(),
            ))
            .set_parent(taken.parent.get())
            .id();
        if taken.filthy.is_some() {
            commands.entity(left_over_entity).insert(Filthy);
        }
        if let Some(&taken_pos) = taken.pos {
            commands.entity(left_over_entity).insert(taken_pos);
        }

        commands
            .entity(taken.entity)
            .insert(allowed_amount)
            .remove::<Pos>()
            .set_parent(container_entity);
    }

    fn take_all(commands: &mut Commands, container_entity: Entity, taken_entity: Entity) {
        commands
            .entity(container_entity)
            .add_children(&[taken_entity]);
        commands.entity(taken_entity).remove::<Pos>();
    }

    pub(crate) fn move_item(
        &self,
        commands: &mut Commands,
        message_writer: &mut MessageWriter,
        subzone_level_entities: &SubzoneLevelEntities,
        location: &mut Location,
        moved: &ItemItem,
        to: Nbor,
    ) -> ActorImpact {
        if let Some(from) = moved.pos {
            let offset = *from - *self.pos;
            let potentially_valid = HorizontalDirection::try_from(offset).is_ok()
                || matches!(offset.level, LevelOffset { h: -1 | 1 });
            if !potentially_valid {
                message_writer.str("Too far to move").send_error();
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

        message_writer
            .subject(self.subject())
            .verb(if dump { "drop" } else { "move" }, "s")
            .extend(moved.fragments())
            .send_info();

        let Some(new_parent) = subzone_level_entities.get(SubzoneLevel::from(to)) else {
            message_writer
                .str("Subzone not found when moving an item")
                .send_error();
            return self.no_impact();
        };
        commands
            .entity(moved.entity)
            .insert((Visibility::default(), to))
            .set_parent(new_parent);
        location.move_(moved.entity, to);
        self.impact_from_duration(Duration::SECOND, StaminaCost::NEUTRAL)
    }

    pub(crate) fn start_craft(
        &self,
        commands: &mut Commands,
        message_writer: &mut MessageWriter,
        next_player_action_state: &mut NextState<PlayerActionState>,
        spawner: &mut TileSpawner,
        infos: &Infos,
        subzone_level_entities: &SubzoneLevelEntities,
        item_amounts: &mut Query<&mut Amount>,
        start_craft: &StartCraft,
    ) -> ActorImpact {
        let pos = self.pos.horizontal_nbor(start_craft.target);
        let Some(parent_entity) = subzone_level_entities.get(SubzoneLevel::from(pos)) else {
            message_writer
                .str("Subzone not found when starting to craft")
                .send_error();
            return self.no_impact();
        };

        for AlternativeSituation {
            item_entities,
            required,
            ..
        } in start_craft.recipe_situation.consumed_components()
        {
            //println!("Consume {required} from {item_entities:?}:");
            let mut missing = *required;
            for &item_entity in item_entities {
                let mut item_amount = item_amounts
                    .get_mut(item_entity)
                    .expect("Consumed component items should be found");
                if item_amount.0 <= missing {
                    //println!(" - Consume {item_entity} fully ({:?}x)", item_amount.0);
                    commands.entity(item_entity).despawn_recursive();
                    missing -= item_amount.0;
                    if missing == 0 {
                        break;
                    }
                } else {
                    //println!(" - Consume {item_entity:?} partially ({}/{})",missing, item_amount.0);
                    item_amount.0 -= missing;
                    break;
                }
            }
        }

        let item = spawner.spawn_craft(
            infos,
            parent_entity,
            pos,
            start_craft.recipe_situation.recipe_id().clone(),
        );

        next_player_action_state.set(PlayerActionState::Crafting { item });

        self.no_impact()
    }

    pub(crate) fn continue_craft(
        &self,
        commands: &mut Commands,
        message_writer: &mut MessageWriter,
        next_player_action_state: &mut NextState<PlayerActionState>,
        spawner: &mut TileSpawner,
        infos: &Infos,
        crafts: &mut Query<(Item, &mut Craft)>,
        craft_entity: Entity,
    ) -> ActorImpact {
        let (item, mut craft) = crafts.get_mut(craft_entity).expect("Craft should be found");

        let crafting_progress = Duration::SECOND * 3;

        craft.work(crafting_progress);
        if craft.finished() {
            message_writer.you("finish").add("your craft").send(
                PlayerActionState::Crafting { item: craft_entity }.severity_finishing(),
                false,
            );
            let parent = item.parent.get();
            let pos = *item.pos.unwrap_or(self.pos);
            let amount = *item.amount;
            let cdda_item = CddaItem::from(craft.object_id.clone());
            commands.entity(item.entity).despawn_recursive();
            _ = spawner.spawn_item(infos, parent, pos, &cdda_item, amount);
            next_player_action_state.set(PlayerActionState::Normal);
        } else {
            let percent_progress = craft.percent_progress();
            let color = text_color_expect_full(percent_progress / 100.0);
            let percent_progress = format!("{percent_progress:.1}");
            let time_left = craft.time_left().short_format();
            message_writer
                .str("Craft:")
                .push(Fragment::colorized(percent_progress, color))
                .add("% progress -")
                .push(Fragment::colorized(time_left, color))
                .add("left")
                .send(Severity::Info, true);
        }
        self.impact_from_duration(crafting_progress, StaminaCost::NEUTRAL)
    }

    pub(crate) fn examine_item(
        &self,
        message_writer: &mut MessageWriter,
        infos: &Infos,
        item: &ItemItem,
    ) -> ActorImpact {
        if let Some(item_info) = infos.try_common_item_info(&item.definition.id) {
            if let Some(description) = &item_info.description {
                message_writer
                    .str(&**match description {
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
        self.no_impact()
    }

    pub(crate) fn change_pace(
        &self,
        commands: &mut Commands,
        change_pace: ChangePace,
    ) -> ActorImpact {
        commands
            .entity(self.entity)
            .insert(self.walking_mode.switch(change_pace));
        self.no_impact()
    }
}
