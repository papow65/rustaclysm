use crate::prelude::*;
use bevy::prelude::*;
use std::time::Instant;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn egible_character(
    envir: Envir,
    mut timeouts: ResMut<Timeouts>,
    actors: Query<Actor>,
    players: Query<(), With<Player>>,
) -> Option<Entity> {
    let egible_entities = actors
        .iter()
        .filter(|a| envir.is_accessible(*a.pos) || players.get(a.entity).is_ok())
        .map(|a| a.entity)
        .collect::<Vec<Entity>>();
    timeouts.next(&egible_entities)
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn plan_action(
    In(active_actor): In<Option<Entity>>,
    mut commands: Commands,
    mut message_writer: EventWriter<Message>,
    mut healing_writer: EventWriter<ActionEvent<Healing>>,
    mut player_action_state: ResMut<PlayerActionState>,
    mut envir: Envir,
    clock: Clock,
    mut instruction_queue: ResMut<InstructionQueue>,
    actors: Query<Actor>,
    factions: Query<(&Pos, &Faction), With<Life>>,
    mut players: Query<(), With<Player>>,
) -> Option<(Entity, PlannedAction)> {
    let start = Instant::now();

    let Some(active_actor) = active_actor else {
        eprintln!("No egible characters!");
        return None;
    };

    let factions = &factions.iter().map(|(p, f)| (*p, f)).collect::<Vec<_>>();
    let actor = actors.get(active_actor).unwrap();
    let enemies = actor.faction.enemies(&envir, &clock, factions, &actor);
    let action = if players.get_mut(active_actor).is_ok() {
        player_action_state.plan_action(
            &mut message_writer,
            &mut healing_writer,
            &mut envir,
            &mut instruction_queue,
            &actor,
            clock.time(),
            &enemies,
        )?
    } else {
        let strategy = actor.faction.strategize(&envir, factions, &enemies, &actor);
        if let Some(last_enemy) = strategy.last_enemy {
            commands.entity(actor.entity).insert(last_enemy);
        }
        println!(
            "{} at {:?} plans {:?} and does {:?}",
            actor.name.single().text,
            actor.pos,
            strategy.intent,
            strategy.action
        );

        strategy.action
    };

    log_if_slow("plan_action", start);

    Some((actor.entity, action))
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn send_action_event(
    In(option): In<Option<(Entity, PlannedAction)>>,
    mut stay_writer: EventWriter<ActionEvent<Stay>>,
    mut step_writer: EventWriter<ActionEvent<Step>>,
    mut attack_writer: EventWriter<ActionEvent<Attack>>,
    mut smash_writer: EventWriter<ActionEvent<Smash>>,
    mut pulp_writer: EventWriter<ActionEvent<Pulp>>,
    mut close_writer: EventWriter<ActionEvent<Close>>,
    mut wield_writer: EventWriter<ActionEvent<ItemAction<Wield>>>,
    mut unwield_writer: EventWriter<ActionEvent<ItemAction<Unwield>>>,
    mut pickup_writer: EventWriter<ActionEvent<ItemAction<Pickup>>>,
    mut move_item_writer: EventWriter<ActionEvent<ItemAction<MoveItem>>>,
    mut examine_item_writer: EventWriter<ActionEvent<ItemAction<ExamineItem>>>,
    mut change_pace_writer: EventWriter<ActionEvent<ChangePace>>,
) {
    let start = Instant::now();

    let Some((actor_entity, action)) = option else {
        // No egible characters
        return;
    };

    match action {
        PlannedAction::Stay { duration } => {
            stay_writer.send(ActionEvent::new(actor_entity, Stay { duration }));
        }
        PlannedAction::Step { to } => {
            step_writer.send(ActionEvent::new(actor_entity, Step { to }));
        }
        PlannedAction::Attack { target } => {
            attack_writer.send(ActionEvent::new(actor_entity, Attack { target }));
        }
        PlannedAction::Smash { target } => {
            smash_writer.send(ActionEvent::new(actor_entity, Smash { target }));
        }
        PlannedAction::Pulp { target } => {
            //eprintln!("Pulp action!");
            pulp_writer.send(ActionEvent::new(actor_entity, Pulp { target }));
        }
        PlannedAction::Close { target } => {
            close_writer.send(ActionEvent::new(actor_entity, Close { target }));
        }
        PlannedAction::Wield { item } => {
            wield_writer.send(ActionEvent::new(actor_entity, ItemAction::new(item, Wield)));
        }
        PlannedAction::Unwield { item } => {
            unwield_writer.send(ActionEvent::new(
                actor_entity,
                ItemAction::new(item, Unwield),
            ));
        }
        PlannedAction::Pickup { item } => {
            pickup_writer.send(ActionEvent::new(
                actor_entity,
                ItemAction::new(item, Pickup),
            ));
        }
        PlannedAction::MoveItem { item, to } => {
            move_item_writer.send(ActionEvent::new(
                actor_entity,
                ItemAction::new(item, MoveItem { to }),
            ));
        }
        PlannedAction::ExamineItem { item } => {
            examine_item_writer.send(ActionEvent::new(
                actor_entity,
                ItemAction::new(item, ExamineItem),
            ));
        }
        PlannedAction::ChangePace => {
            change_pace_writer.send(ActionEvent::new(actor_entity, ChangePace));
        }
    };

    log_if_slow("send_action_event", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn check_action_plan_amount(
    mut stay_reader: EventReader<ActionEvent<Stay>>,
    mut step_reader: EventReader<ActionEvent<Step>>,
    mut attack_reader: EventReader<ActionEvent<Attack>>,
    mut smash_reader: EventReader<ActionEvent<Smash>>,
    mut pulp_reader: EventReader<ActionEvent<Pulp>>,
    mut close_reader: EventReader<ActionEvent<Close>>,
    mut wield_reader: EventReader<ActionEvent<ItemAction<Wield>>>,
    mut unwield_reader: EventReader<ActionEvent<ItemAction<Unwield>>>,
    mut pickup_reader: EventReader<ActionEvent<ItemAction<Pickup>>>,
    mut move_item_reader: EventReader<ActionEvent<ItemAction<MoveItem>>>,
    mut examine_item_reader: EventReader<ActionEvent<ItemAction<ExamineItem>>>,
    mut change_pace_reader: EventReader<ActionEvent<ChangePace>>,
) {
    let all = stay_reader
        .read()
        .map(|a| format!("{a:?}"))
        .chain(step_reader.read().map(|a| format!("{a:?}")))
        .chain(attack_reader.read().map(|a| format!("{a:?}")))
        .chain(smash_reader.read().map(|a| format!("{a:?}")))
        .chain(pulp_reader.read().map(|a| format!("{a:?}")))
        .chain(close_reader.read().map(|a| format!("{a:?}")))
        .chain(wield_reader.read().map(|a| format!("{a:?}")))
        .chain(unwield_reader.read().map(|a| format!("{a:?}")))
        .chain(pickup_reader.read().map(|a| format!("{a:?}")))
        .chain(move_item_reader.read().map(|a| format!("{a:?}")))
        .chain(examine_item_reader.read().map(|a| format!("{a:?}")))
        .chain(change_pace_reader.read().map(|a| format!("{a:?}")))
        .collect::<Vec<_>>();

    assert!(all.len() <= 1, "Multiple actions: {all:?}");
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn single_action<A: Action>(
    mut action_reader: EventReader<ActionEvent<A>>,
) -> ActionEvent<A> {
    action_reader.read().next().cloned().expect("Single event")
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn perform_stay(In(stay): In<ActionEvent<Stay>>, actors: Query<Actor>) -> Impact {
    stay.actor(&actors).stay(&stay.action)
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_step(
    In(step): In<ActionEvent<Step>>,
    mut commands: Commands,
    mut message_writer: EventWriter<Message>,
    mut toggle_writer: EventWriter<TerrainEvent<Toggle>>,
    mut envir: Envir,
    actors: Query<Actor>,
) -> Impact {
    step.actor(&actors).step(
        &mut commands,
        &mut message_writer,
        &mut toggle_writer,
        &mut envir,
        &step.action,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_attack(
    In(attack): In<ActionEvent<Attack>>,
    mut message_writer: EventWriter<Message>,
    mut damage_writer: EventWriter<ActionEvent<Damage>>,
    envir: Envir,
    infos: Res<Infos>,
    hierarchy: Hierarchy,
    actors: Query<Actor>,
) -> Impact {
    attack.actor(&actors).attack(
        &mut message_writer,
        &mut damage_writer,
        &envir,
        &infos,
        &hierarchy,
        &attack.action,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_smash(
    In(smash): In<ActionEvent<Smash>>,
    mut message_writer: EventWriter<Message>,
    mut damage_writer: EventWriter<TerrainEvent<Damage>>,
    envir: Envir,
    infos: Res<Infos>,
    hierarchy: Hierarchy,
    actors: Query<Actor>,
) -> Impact {
    smash.actor(&actors).smash(
        &mut message_writer,
        &mut damage_writer,
        &envir,
        &infos,
        &hierarchy,
        &smash.action,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_pulp(
    In(pulp): In<ActionEvent<Pulp>>,
    mut message_writer: EventWriter<Message>,
    mut corpse_damage_writer: EventWriter<CorpseEvent<Damage>>,
    envir: Envir,
    infos: Res<Infos>,
    hierarchy: Hierarchy,
    actors: Query<Actor>,
) -> Impact {
    pulp.actor(&actors).pulp(
        &mut message_writer,
        &mut corpse_damage_writer,
        &envir,
        &infos,
        &hierarchy,
        &pulp.action,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_close(
    In(close): In<ActionEvent<Close>>,
    mut message_writer: EventWriter<Message>,
    mut toggle_writer: EventWriter<TerrainEvent<Toggle>>,
    envir: Envir,
    actors: Query<Actor>,
) -> Impact {
    close.actor(&actors).close(
        &mut message_writer,
        &mut toggle_writer,
        &envir,
        &close.action,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_wield(
    In(wield): In<ActionEvent<ItemAction<Wield>>>,
    mut commands: Commands,
    mut message_writer: EventWriter<Message>,
    mut location: ResMut<Location>,
    hierarchy: Hierarchy,
    actors: Query<Actor>,
    items: Query<Item>,
) -> Impact {
    wield.actor(&actors).wield(
        &mut commands,
        &mut message_writer,
        &mut location,
        &hierarchy,
        &wield.action.item(&items),
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_unwield(
    In(unwield): In<ActionEvent<ItemAction<Unwield>>>,
    mut commands: Commands,
    mut message_writer: EventWriter<Message>,
    mut location: ResMut<Location>,
    hierarchy: Hierarchy,
    actors: Query<Actor>,
    items: Query<Item>,
) -> Impact {
    unwield.actor(&actors).unwield(
        &mut commands,
        &mut message_writer,
        &mut location,
        &hierarchy,
        &unwield.action.item(&items),
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_pickup(
    In(pickup): In<ActionEvent<ItemAction<Pickup>>>,
    mut commands: Commands,
    mut message_writer: EventWriter<Message>,
    mut location: ResMut<Location>,
    hierarchy: Hierarchy,
    actors: Query<Actor>,
    items: Query<Item>,
) -> Impact {
    pickup.actor(&actors).pickup(
        &mut commands,
        &mut message_writer,
        &mut location,
        &hierarchy,
        &pickup.action.item(&items),
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_move_item(
    In(move_item): In<ActionEvent<ItemAction<MoveItem>>>,
    mut commands: Commands,
    mut message_writer: EventWriter<Message>,
    subzone_level_entities: Res<SubzoneLevelEntities>,
    mut location: ResMut<Location>,
    actors: Query<Actor>,
    items: Query<Item>,
) -> Impact {
    move_item.actor(&actors).move_item(
        &mut commands,
        &mut message_writer,
        &subzone_level_entities,
        &mut location,
        &move_item.action.item(&items),
        move_item.action.change.to,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_examine_item(
    In(examine_item): In<ActionEvent<ItemAction<ExamineItem>>>,
    mut message_writer: EventWriter<Message>,
    infos: Res<Infos>,
    items: Query<Item>,
) {
    ActorItem::examine_item(
        &mut message_writer,
        &infos,
        &examine_item.action.item(&items),
    );
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_change_pace(
    In(change_pace): In<ActionEvent<ChangePace>>,
    mut commands: Commands,
    actors: Query<Actor>,
) {
    change_pace.actor(&actors).change_pace(&mut commands);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn proces_impact(
    In(impact): In<Impact>,
    mut message_writer: EventWriter<Message>,
    mut stamina_impact_events: EventWriter<ActionEvent<StaminaImpact>>,
    mut timeouts: ResMut<Timeouts>,
    players: Query<Entity, With<Player>>,
) {
    let start = Instant::now();

    impact.check_validity();
    let Some(stamina_impact) = impact.stamina_impact else {
        if players.get(impact.actor_entity).is_err() {
            message_writer.send(Message::error(Phrase::new("NPC action failed")));
            // To prevent the application hanging on failed NPC actions, we add a small timeout
            timeouts.add(impact.actor_entity, Milliseconds(1000));
        }

        return;
    };

    timeouts.add(impact.actor_entity, impact.timeout);

    stamina_impact_events.send(ActionEvent::new(impact.actor_entity, stamina_impact));

    log_if_slow("proces_impact", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_stamina(
    mut stamina_impact_events: EventReader<ActionEvent<StaminaImpact>>,
    mut staminas: Query<&mut Stamina>,
) {
    let start = Instant::now();

    assert!(
        stamina_impact_events.len() <= 1,
        "Multiple stamina impact events: {:?}",
        stamina_impact_events.read().collect::<Vec<_>>()
    );

    for stamina_impact_event in stamina_impact_events.read() {
        if let Ok(mut stamina) = staminas.get_mut(stamina_impact_event.actor_entity) {
            stamina.apply(stamina_impact_event.action);
        }
    }

    log_if_slow("update_stamina", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn toggle_doors(
    mut commands: Commands,
    mut toggle_reader: EventReader<TerrainEvent<Toggle>>,
    infos: Res<Infos>,
    mut spawner: Spawner,
    mut visualization_update: ResMut<VisualizationUpdate>,
    terrain: Query<(Entity, &ObjectDefinition, &Pos, &Parent)>,
) {
    let start = Instant::now();

    for toggle in toggle_reader.read() {
        let (entity, definition, &pos, parent) = terrain.get(toggle.terrain_entity).expect("Found");

        commands.entity(entity).despawn_recursive();
        let terrain_info = infos.terrain(&definition.id).expect("Valid terrain");
        let toggled_id = match toggle.change {
            Toggle::Open => terrain_info.open.as_ref().expect("Openable"),
            Toggle::Close => terrain_info.close.as_ref().expect("Closeable"),
        };
        spawner.spawn_terrain(&infos, parent.get(), pos, toggled_id);
        *visualization_update = VisualizationUpdate::Forced;
    }

    log_if_slow("toggle_doors", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_damaged_characters(
    mut commands: Commands,
    mut message_writer: EventWriter<Message>,
    mut damage_reader: EventReader<ActionEvent<Damage>>,
    clock: Clock,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    mut characters: Query<
        (&ObjectName, &mut Health, &mut Transform, Option<&Player>),
        (With<Faction>, With<Life>),
    >,
) {
    let start = Instant::now();

    for damage in damage_reader.read() {
        let (name, mut health, mut transform, player) = characters
            .get_mut(damage.actor_entity)
            .expect("Actor found");
        let evolution = health.lower(&damage.action);
        if health.0.is_zero() {
            message_writer.send(Message::warn(
                damage
                    .action
                    .attacker
                    .clone()
                    .verb("kill", "s")
                    .push(name.single()),
            ));
            transform.rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)
                * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
            commands
                .entity(damage.actor_entity)
                .insert(Corpse)
                .insert(CorpseRaise {
                    at: clock.time() + Milliseconds::EIGHT_HOURS,
                })
                .insert(ObjectName::corpse())
                .insert(Integrity(Limited::full(400)))
                .remove::<Life>()
                .remove::<Obstacle>();

            if player.is_some() {
                next_gameplay_state.set(GameplayScreenState::Death);
            }
        } else {
            message_writer.send({
                let begin = damage
                    .action
                    .attacker
                    .clone()
                    .verb("hit", "s")
                    .push(name.single());

                Message::warn(if evolution.changed() {
                    begin.add(format!(
                        "for {} ({} -> {})",
                        evolution.change_abs(),
                        evolution.before,
                        evolution.after
                    ))
                } else {
                    begin.add("but it has no effect")
                })
            });
        }
    }

    log_if_slow("update_damaged_characters", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_healed_characters(
    mut message_writer: EventWriter<Message>,
    mut healing_reader: EventReader<ActionEvent<Healing>>,
    mut actors: ParamSet<(
        Query<&mut Health, (With<Faction>, With<Life>)>,
        Query<Actor, (With<Faction>, With<Life>)>,
    )>,
) {
    let start = Instant::now();

    for healing in healing_reader.read() {
        let mut healths = actors.p0();
        let mut health = healths.get_mut(healing.actor_entity).expect("Actor found");
        let evolution = health.raise(&healing.action);
        if evolution.changed() {
            let actors = actors.p1();
            let actor = actors.get(healing.actor_entity).expect("Actor found");
            message_writer.send({
                let begin = actor.subject().verb("heal", "s");

                Message::info(if evolution.change_abs() == 1 {
                    begin.add("a bit")
                } else {
                    begin.add(format!(
                        "for {} ({} -> {})",
                        evolution.change_abs(),
                        evolution.before,
                        evolution.after
                    ))
                })
            });
        }
    }

    log_if_slow("update_healed_characters", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_damaged_corpses(
    mut commands: Commands,
    mut message_writer: EventWriter<Message>,
    mut damage_reader: EventReader<CorpseEvent<Damage>>,
    mut corpses: Query<(&ObjectName, &mut Integrity), With<Corpse>>,
) {
    let start = Instant::now();

    //eprintln!("{} corpses found", corpses.iter().len());

    for damage in damage_reader.read() {
        let (name, mut integrity) = corpses.get_mut(damage.corpse_entity).expect("Corpse found");
        integrity.lower(&damage.change);

        message_writer.send(Message::info(
            damage
                .change
                .attacker
                .clone()
                .verb("pulp", "s")
                .push(name.single()),
        ));

        if integrity.0.is_zero() {
            message_writer.send(Message::info(
                Phrase::from_name(name).add("is thoroughly pulped"),
            ));

            commands
                .entity(damage.corpse_entity)
                .remove::<Integrity>()
                .remove::<CorpseRaise>();
        }
    }

    log_if_slow("update_damaged_corpses", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_corpses(
    mut commands: Commands,
    clock: Clock,
    infos: Res<Infos>,
    mut corpse_raises: Query<(Entity, &CorpseRaise, &mut Transform)>,
) {
    let start = Instant::now();

    for (corpse, raise, mut transform) in &mut corpse_raises {
        if raise.at <= clock.time() {
            transform.rotation = Quat::IDENTITY;

            let definition = ObjectDefinition {
                category: ObjectCategory::Character,
                id: ObjectId::new("mon_zombie"),
            };
            let character_info = infos.character(&definition.id).expect("Info available");
            let object_name = ObjectName::new(character_info.name.clone(), Faction::Zombie.color());
            let health = Health(Limited::full(character_info.hp.unwrap_or(60) as u16));

            commands
                .entity(corpse)
                .insert(definition)
                .insert(object_name)
                .insert(Faction::Zombie)
                .insert(Life)
                .insert(health)
                .insert(Stamina::Unlimited)
                .insert(WalkingMode::Running)
                .insert(Obstacle)
                .remove::<Corpse>()
                .remove::<CorpseRaise>()
                .remove::<Integrity>();
        }
    }

    log_if_slow("update_corpses", start);
}

/** For terrain and furniture */
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_damaged_terrain(
    mut commands: Commands,
    mut message_writer: EventWriter<Message>,
    mut damage_reader: EventReader<TerrainEvent<Damage>>,
    mut spawner: Spawner,
    mut visualization_update: ResMut<VisualizationUpdate>,
    infos: Res<Infos>,
    mut terrain: Query<(
        Entity,
        &Pos,
        &ObjectName,
        &mut Integrity,
        &ObjectDefinition,
        &Parent,
    )>,
) {
    let start = Instant::now();

    for damage in damage_reader.read() {
        let (terrain, &pos, name, mut integrity, definition, parent) = terrain
            .get_mut(damage.terrain_entity)
            .expect("Terrain or furniture found");
        let evolution = integrity.lower(&damage.change);
        if integrity.0.is_zero() {
            message_writer.send(Message::warn(
                damage
                    .change
                    .attacker
                    .clone()
                    .verb("break", "s")
                    .push(name.single()),
            ));
            commands.entity(terrain).despawn_recursive();
            spawner.spawn_smashed(&infos, parent.get(), pos, definition);
            *visualization_update = VisualizationUpdate::Forced;
        } else {
            message_writer.send({
                let begin = damage
                    .change
                    .attacker
                    .clone()
                    .verb("hit", "s")
                    .push(name.single());

                Message::warn(if evolution.changed() {
                    begin.add(format!(
                        "for {} ({} -> {})",
                        evolution.change_abs(),
                        evolution.before,
                        evolution.after
                    ))
                } else {
                    begin.add("but it has no effect")
                })
            });
        }
    }

    log_if_slow("update_damaged_items", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn combine_items(
    mut commands: Commands,
    hierarchy: Hierarchy,
    moved_items: Query<Item, (Changed<Parent>, Without<ContainerLimits>)>,
) {
    let start = Instant::now();

    let mut all_merged = Vec::new();

    for moved in &moved_items {
        if moved.definition.category == ObjectCategory::Item && !all_merged.contains(&moved.entity)
        {
            let mut merges = vec![moved.entity];
            let mut total_amount = &Amount(0) + moved.amount;

            for sibling in hierarchy.items_in(moved.parent.get()) {
                // Note that the positions may differ when the parents are the same.
                if sibling.entity != moved.entity
                    && sibling.definition == moved.definition
                    && sibling.pos == moved.pos
                    && sibling.filthy == moved.filthy
                    && !all_merged.contains(&sibling.entity)
                {
                    merges.push(sibling.entity);
                    total_amount = &total_amount + sibling.amount;
                    all_merged.push(sibling.entity);
                }
            }

            if 1 < merges.len() {
                let keep = *merges.iter().max().unwrap();

                /*println!(
                    "Combined {} with {} others to {}",
                    Phrase::from_fragments(moved_name.as_item(moved_amount, moved_filthy)),
                    merges.len() - 1,
                    Phrase::from_fragments(moved_name.as_item(Some(&total_amount), moved_filthy)),
                );*/

                commands.entity(keep).insert(total_amount);

                for merge in merges {
                    if merge != keep {
                        commands.entity(merge).despawn_recursive();
                    }
                }
            }
        }
    }

    log_if_slow("update_damaged_items", start);
}
