use crate::actor::phrases::{
    AttackNothing, CantClose, CantCloseOn, CraftProgressLeft, CrashInto, Drop, HaltAtTheLedge,
    IsTooExhaustedTo, Move, PickUp, PulpNothing, SmashInvalid, SubzoneNotFoundWhileMovingAnItem,
    TooFarToMove, YouCant, YouFinish, YouSleepFor,
};
use crate::{
    ActorEvent, ActorImpact, Amount, Aquatic, Attack, BaseSpeed, BodyContainers, Breath,
    ChangePace, Clock, Close, Collision, Consumed, Container, CorpseEvent, Craft, Damage, Envir,
    Faction, Healing, HealingDuration, Health, InPocket, Item, ItemHierarchy, ItemItem, LastEnemy,
    Life, LogMessageWriter, Melee, ObjectName, ObjectOn, Peek, Phrase, Player, PlayerActionState,
    PlayerWielded, Pulp, Smash, Stamina, StaminaCost, StartCraft, Step, Subject, TerrainEvent,
    Tile, TileSpawner, Toggle, WalkingMode,
};
use bevy::ecs::query::{QueryData, With};
use bevy::prelude::{
    Commands, Entity, Message, MessageWriter, NextState, Query, Transform, Visibility, error,
};
use cdda_json_files::CddaItem;
use either::Either;
use gameplay_location::{HorizontalDirection, LevelOffset, LocationCache, Nbor, Pos};
use gameplay_model::LastSeen;
use units::{Distance, Duration, Speed};
use util::Maybe;

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

impl ActorItem<'_, '_> {
    pub(crate) fn subject(&self) -> Subject {
        if self.player.is_some() {
            Subject::You
        } else {
            Subject::Other(Phrase::from_fragment(self.name.single(*self.pos)))
        }
    }

    pub(crate) const fn speed(&self) -> Speed {
        self.base_speed
            .speed(self.walking_mode, self.stamina.breath())
    }

    const fn high_speed(&self) -> Option<Speed> {
        match self.stamina.breath() {
            Breath::Normal | Breath::AlmostWinded => {
                Some(self.base_speed.speed(&WalkingMode::Running, Breath::Normal))
            }
            Breath::Winded => None,
        }
    }

    const fn hands<'a>(&self, hierarchy: &'a ItemHierarchy) -> Container<'a> {
        Container::new(
            self.body_containers.expect("Body containers present").hands,
            hierarchy,
        )
    }

    const fn clothing<'a>(&self, hierarchy: &'a ItemHierarchy) -> Container<'a> {
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

    pub(crate) fn stay_duration(&self) -> Duration {
        Distance::ADJACENT / 2 / self.high_speed().unwrap_or_else(|| self.speed())
    }

    pub(crate) fn stay(&self) -> ActorImpact {
        self.impact_from_duration(self.stay_duration(), StaminaCost::STANDING_REST)
    }

    pub(crate) fn sleep(
        &self,
        transient_message_writer: &mut LogMessageWriter<PlayerActionState>,
        healing_writer: &mut MessageWriter<'_, ActorEvent<Healing>>,
        player_action_state: &PlayerActionState,
        clock: &Clock,
        healing_durations: &mut Query<&mut HealingDuration>,
    ) -> ActorImpact {
        const SLEEP_DURATION: Duration = Duration::MINUTE;

        let mut healing_duration = healing_durations
            .get_mut(self.entity)
            .expect("Actor entity should be found");

        let healing_amount = healing_duration.heal(SLEEP_DURATION);
        healing_writer.write(ActorEvent::new(
            self.entity,
            Healing {
                amount: healing_amount as u16,
            },
        ));

        if let PlayerActionState::Sleeping { from } = player_action_state {
            transient_message_writer.send_transient(
                YouSleepFor {
                    total_duration: clock.time() - *from,
                },
                player_action_state.clone(),
            );
        } else {
            error!("Unexpected {player_action_state:?} while sleeping");
        }

        self.impact_from_duration(SLEEP_DURATION, StaminaCost::LYING_REST)
    }

    pub(crate) fn step(
        &self,
        commands: &mut Commands,
        message_writer: &mut LogMessageWriter,
        toggle_writer: &mut MessageWriter<TerrainEvent<Toggle>>,
        envir: &mut Envir,
        step: &Step,
    ) -> ActorImpact {
        let from = *self.pos;
        let to = envir.get_nbor(from, step.to).expect("Valid pos");

        match envir.collide(from, to, true) {
            Collision::Pass => {
                commands.entity(self.entity).insert(to);
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
                message_writer.send(CrashInto {
                    subject: self.subject(),
                    obstacle,
                    to,
                });
                self.no_impact()
            }
            Collision::Ledged => {
                message_writer.send(HaltAtTheLedge {
                    subject: self.subject(),
                });
                self.no_impact()
            }
            Collision::Opened(door) => {
                toggle_writer.write(TerrainEvent {
                    terrain_entity: door,
                    change: Toggle::Open,
                });
                self.impact_from_duration(Duration::SECOND, StaminaCost::NEUTRAL)
            }
        }
    }

    fn damage<E: Message, N>(
        &self,
        damage_writer: &mut MessageWriter<E>,
        hierarchy: &ItemHierarchy,
        damaged: Entity,
        new: N,
    ) -> ActorImpact
    where
        N: Fn(Entity, Damage) -> E,
    {
        let mut melee_weapon = None;
        if let Some(body_containers) = self.body_containers {
            let mut hands_children = hierarchy.items_in_pocket(body_containers.hands);
            if let Some(weapon) = hands_children.next() {
                melee_weapon = Some(weapon.common_info);
            }
        }

        let damage = Damage {
            attacker: self.subject(),
            amount: self.melee.damage(melee_weapon),
        };
        damage_writer.write(new(damaged, damage));

        self.impact_from_duration(Duration::SECOND, StaminaCost::HEAVY)
    }

    pub(crate) fn attack(
        &self,
        message_writer: &mut LogMessageWriter,
        damage_writer: &mut MessageWriter<ActorEvent<Damage>>,
        envir: &Envir,
        hierarchy: &ItemHierarchy,
        attack: &Attack,
    ) -> ActorImpact {
        if self.stamina.breath() == Breath::Winded {
            message_writer.send(IsTooExhaustedTo {
                subject: self.subject(),
                verb: "attack",
            });
            return self.no_impact();
        }

        let target = envir.get_nbor(*self.pos, attack.target).expect("Valid pos");

        if let Some((defender, _)) = envir.find_character(target) {
            self.damage(damage_writer, hierarchy, defender, ActorEvent::new)
        } else {
            message_writer.send(AttackNothing {
                subject: self.subject(),
            });
            self.no_impact()
        }
    }

    pub(crate) fn smash(
        &self,
        message_writer: &mut LogMessageWriter,
        damage_writer: &mut MessageWriter<TerrainEvent<Damage>>,
        envir: &Envir,
        hierarchy: &ItemHierarchy,
        smash: &Smash,
    ) -> ActorImpact {
        if self.stamina.breath() == Breath::Winded {
            message_writer.send(IsTooExhaustedTo {
                subject: self.subject(),
                verb: "smash",
            });
            return self.no_impact();
        }

        let target = envir.get_nbor(*self.pos, smash.target).expect("Valid pos");

        let stair_pos = Pos::new(target.x, self.pos.level, target.z);
        if self.pos.level.up() == Some(target.level) && envir.stairs_up_to(stair_pos).is_none() {
            message_writer.send(SmashInvalid {
                subject: self.subject(),
                object: "the ceiling",
            });
            self.no_impact()
        } else if self.pos.level.down() == Some(target.level)
            && envir.stairs_down_to(stair_pos).is_none()
        {
            message_writer.send(SmashInvalid {
                subject: self.subject(),
                object: "the floor",
            });
            self.no_impact()
        } else if let Some(smashable) = envir.find_smashable(target) {
            self.damage(damage_writer, hierarchy, smashable, TerrainEvent::new)
        } else {
            message_writer.send(SmashInvalid {
                subject: self.subject(),
                object: "nothing",
            });
            self.no_impact()
        }
    }

    pub(crate) fn pulp(
        &self,
        message_writer: &mut LogMessageWriter,
        corpse_damage_writer: &mut MessageWriter<CorpseEvent<Damage>>,
        envir: &Envir,
        hierarchy: &ItemHierarchy,
        pulp: &Pulp,
    ) -> ActorImpact {
        if self.stamina.breath() == Breath::Winded {
            message_writer.send(IsTooExhaustedTo {
                subject: self.subject(),
                verb: "pulp",
            });
            return self.no_impact();
        }

        let target = self.pos.horizontal_nbor(pulp.target);

        if let Some(pulpable_entity) = envir.find_pulpable(target) {
            self.damage(
                corpse_damage_writer,
                hierarchy,
                pulpable_entity,
                CorpseEvent::new,
            )
        } else {
            message_writer.send(PulpNothing {
                subject: self.subject(),
            });
            self.no_impact()
        }
    }

    pub(crate) fn peek(
        &self,
        message_writer: &mut LogMessageWriter,
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
                    message_writer.send(IsTooExhaustedTo {
                        subject: self.subject(),
                        verb: "peek",
                    });
                    self.no_impact()
                }
            },
            _ => {
                message_writer.send(YouCant {
                    verb: "peek",
                    direction: "there",
                });
                self.no_impact()
            }
        }
    }

    pub(crate) fn close(
        &self,
        message_writer: &mut LogMessageWriter,
        toggle_writer: &mut MessageWriter<TerrainEvent<Toggle>>,
        envir: &Envir,
        close: &Close,
    ) -> ActorImpact {
        let target = self.pos.horizontal_nbor(close.target);

        if let Some((closeable, closeable_name)) = envir.find_closeable(target) {
            if let Some((_, character)) = envir.find_character(target) {
                message_writer.send(CantCloseOn {
                    subject: self.subject(),
                    closeable: closeable_name.single(target),
                    obstacle: character.single(target),
                });
                self.no_impact()
            } else {
                toggle_writer.write(TerrainEvent {
                    terrain_entity: closeable,
                    change: Toggle::Close,
                });
                self.impact_from_duration(Duration::SECOND, StaminaCost::NEUTRAL)
            }
        } else {
            let missing = ObjectName::missing();
            let obstacle = envir.find_terrain(target).unwrap_or(&missing);
            message_writer.send(CantClose {
                subject: self.subject(),
                uncloseable: obstacle.single(target),
            });
            self.no_impact()
        }
    }

    pub(crate) fn wield(
        &self,
        commands: &mut Commands,
        message_writer: &mut LogMessageWriter,
        hierarchy: &ItemHierarchy,
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
        message_writer: &mut LogMessageWriter,
        hierarchy: &ItemHierarchy,
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
        message_writer: &mut LogMessageWriter,
        hierarchy: &ItemHierarchy,
        item: &ItemItem,
    ) -> ActorImpact {
        self.take(commands, message_writer, &self.clothing(hierarchy), item)
    }

    fn take(
        &self,
        commands: &mut Commands,
        message_writer: &mut LogMessageWriter,
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
        }
        // TODO check position of root item

        if let Ok(allowed_amount) = target.check_add(
            message_writer,
            self.subject(),
            taken.containable,
            *taken.amount,
        ) {
            message_writer.send(PickUp {
                subject: self.subject(),
                taken: taken.fragments().collect(),
            });

            if &allowed_amount < taken.amount {
                Self::take_some(commands, target.in_pocket, allowed_amount, taken);
            } else {
                Self::take_all(commands, target.in_pocket, taken.entity);
            }
            self.impact_from_duration(Duration::SECOND, StaminaCost::NEUTRAL)
        } else {
            self.no_impact()
        }
    }

    fn take_some(
        commands: &mut Commands,
        to_in_pocket: InPocket,
        allowed_amount: Amount,
        taken: &ItemItem,
    ) {
        let left_over_amount = taken.amount - &allowed_amount;
        //trace!("{:?}", &split_amount);
        //trace!("{:?}", (&allowed_amount);
        //trace!("{:?}", (&left_over_amount);

        // The new entity, left where the old entity was
        commands.spawn((
            taken.common_info.clone(),
            taken.name.clone(),
            left_over_amount,
            taken.containable.clone(),
            LastSeen::Currently,
            Transform::default(),
            Visibility::default(),
            Maybe(taken.on_tile.copied()),
            Maybe(taken.in_pocket.copied()),
            Maybe(taken.filthy.copied()),
            Maybe(taken.pos.copied()),
        ));

        // The old entity, moved to the pocket
        commands
            .entity(taken.entity)
            .insert((allowed_amount, to_in_pocket))
            .remove::<Pos>()
            .remove::<ObjectOn>();
    }

    fn take_all(commands: &mut Commands, to_in_pocket: InPocket, taken_entity: Entity) {
        commands
            .entity(to_in_pocket.pocket_entity)
            .add_related::<InPocket>(&[taken_entity]);
        commands
            .entity(taken_entity)
            .remove::<Pos>()
            .remove::<ObjectOn>();
    }

    pub(crate) fn move_item(
        &self,
        commands: &mut Commands,
        message_writer: &mut LogMessageWriter,
        location: &LocationCache,
        moved: &ItemItem,
        to: Nbor,
        tiles: &Query<Entity, With<Tile>>,
    ) -> ActorImpact {
        if let Some(from) = moved.pos {
            let offset = *from - *self.pos;
            let potentially_valid = HorizontalDirection::try_from(offset).is_ok()
                || matches!(offset.level, LevelOffset { h: -1 | 1 });
            if !potentially_valid {
                message_writer.send(TooFarToMove);
                return self.no_impact();
            }
        }

        let body_containers = self.body_containers.expect("Body containers present");
        let dump = moved
            .in_pocket
            .is_some_and(|in_pocket| body_containers.all().contains(in_pocket));

        // TODO Check for obstacles
        let to = self.pos.raw_nbor(to).expect("Valid position");

        if dump {
            message_writer.send(Drop {
                subject: self.subject(),
                item: moved.fragments().collect(),
            });
        } else {
            message_writer.send(Move {
                subject: self.subject(),
                item: moved.fragments().collect(),
            });
        }

        let Some(tile_entity) = location.get_first(to, tiles) else {
            message_writer.send(SubzoneNotFoundWhileMovingAnItem);
            return self.no_impact();
        };
        commands
            .entity(moved.entity)
            .insert((Visibility::default(), to, ObjectOn { tile_entity }))
            .remove::<InPocket>();
        self.impact_from_duration(Duration::SECOND, StaminaCost::NEUTRAL)
    }

    pub(crate) fn start_craft(
        &self,
        commands: &mut Commands,
        next_player_action_state: &mut NextState<PlayerActionState>,
        spawner: &mut TileSpawner,
        item_amounts: &mut Query<&mut Amount>,
        start_craft: &StartCraft,
    ) -> ActorImpact {
        let pos = self.pos.horizontal_nbor(start_craft.target);

        for Consumed {
            amount,
            from_entities,
        } in start_craft.recipe_situation.consumed_tool_charges()
        {
            //trace!("Consume {required} from {from_entities:?}:");
            let mut missing = amount.get();
            for &consumed_entity in from_entities {
                let mut item_amount = item_amounts
                    .get_mut(consumed_entity)
                    .expect("Consumed tool charges should be found");
                if item_amount.0 <= missing {
                    //trace!(" - Consume {consumed_entity} fully ({:?}x)", item_amount.0);
                    commands.entity(consumed_entity).despawn();
                    missing -= item_amount.0;
                    if missing == 0 {
                        break;
                    }
                } else {
                    //trace!(" - Consume {consumed_entity:?} partially ({}/{})",missing, item_amount.0);
                    item_amount.0 -= missing;
                    break;
                }
            }
        }

        for Consumed {
            amount,
            from_entities,
        } in start_craft.recipe_situation.consumed_components()
        {
            //trace!("Consume {required} from {item_entities:?}:");
            let mut missing = amount.get();
            for &item_entity in from_entities {
                let mut item_amount = item_amounts
                    .get_mut(item_entity)
                    .expect("Consumed component items should be found");
                if item_amount.0 <= missing {
                    //trace!(" - Consume {item_entity} fully ({:?}x)", item_amount.0);
                    commands.entity(item_entity).despawn();
                    missing -= item_amount.0;
                    if missing == 0 {
                        break;
                    }
                } else {
                    //trace!(" - Consume {item_entity:?} partially ({}/{})",missing, item_amount.0);
                    item_amount.0 -= missing;
                    break;
                }
            }
        }

        let item = spawner.spawn_craft(pos, start_craft.recipe_situation.recipe().clone());
        match item {
            Ok(item) => {
                next_player_action_state.set(PlayerActionState::Crafting { item });
            }
            Err(error) => error!("Failed to start crafting: {error:#?}"),
        }

        self.no_impact()
    }

    pub(crate) fn continue_craft(
        &self,
        commands: &mut Commands,
        message_writer: &mut LogMessageWriter,
        transient_message_writer: &mut LogMessageWriter<PlayerActionState>,
        player_action_state: &PlayerActionState,
        next_player_action_state: &mut NextState<PlayerActionState>,
        spawner: &mut TileSpawner,
        crafts: &mut Query<(Item, &mut Craft)>,
        craft_entity: Entity,
    ) -> ActorImpact {
        let (item, mut craft) = crafts.get_mut(craft_entity).expect("Craft should be found");

        let crafting_progress = Duration::SECOND * 3;

        craft.work(crafting_progress);
        if craft.finished() {
            message_writer.send(YouFinish::<true> {
                action: PlayerActionState::Crafting { item: craft_entity },
            });
            let pos = *item.pos.unwrap_or(self.pos);
            let amount = *item.amount;
            commands.entity(item.entity).despawn();
            if let Some(result) = craft.recipe.result.item_info() {
                let cdda_item = CddaItem::new(&result);
                if let Err(error) = match item.parentage().cloned() {
                    Either::Left(child_of) => {
                        spawner.spawn_item(child_of, Some(pos), &cdda_item, amount)
                    }
                    Either::Right(in_pocket) => {
                        spawner.spawn_item(in_pocket, Some(pos), &cdda_item, amount)
                    }
                } {
                    error!("Spawning crafted item failed: {error:#?}");
                }
            }
            next_player_action_state.set(PlayerActionState::Normal);
        } else {
            transient_message_writer.send_transient(
                CraftProgressLeft { craft: &craft },
                player_action_state.clone(),
            );
        }
        self.impact_from_duration(crafting_progress, StaminaCost::NEUTRAL)
    }

    pub(crate) fn examine_item(
        &self,
        message_writer: &mut LogMessageWriter,
        item: &ItemItem,
    ) -> ActorImpact {
        message_writer.send(&item.common_info.description);
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
