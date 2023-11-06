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
    mut healing_writer: EventWriter<ActorEvent<Healing>>,
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
            actor.entity,
            *actor.pos,
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
    mut stay_writer: EventWriter<ActorEvent<Stay>>,
    mut step_writer: EventWriter<ActorEvent<Step>>,
    mut attack_writer: EventWriter<ActorEvent<Attack>>,
    mut smash_writer: EventWriter<ActorEvent<Smash>>,
    mut close_writer: EventWriter<ActorEvent<Close>>,
    mut wield_writer: EventWriter<ActorEvent<Wield>>,
    mut unwield_writer: EventWriter<ActorEvent<Unwield>>,
    mut pickup_writer: EventWriter<ActorEvent<Pickup>>,
    mut move_item_writer: EventWriter<ActorEvent<MoveItem>>,
    mut examine_item_writer: EventWriter<ActorEvent<ExamineItem>>,
    mut change_pace_writer: EventWriter<ActorEvent<ChangePace>>,
) {
    let start = Instant::now();

    let Some((actor_entity, action)) = option else {
        // No egible characters
        return;
    };

    match action {
        PlannedAction::Stay { duration } => {
            stay_writer.send(ActorEvent::new(actor_entity, Stay { duration }));
        }
        PlannedAction::Step { to } => {
            step_writer.send(ActorEvent::new(actor_entity, Step { to }));
        }
        PlannedAction::Attack { target } => {
            attack_writer.send(ActorEvent::new(actor_entity, Attack { target }));
        }
        PlannedAction::Smash { target } => {
            smash_writer.send(ActorEvent::new(actor_entity, Smash { target }));
        }
        PlannedAction::Close { target } => {
            close_writer.send(ActorEvent::new(actor_entity, Close { target }));
        }
        PlannedAction::Wield { item } => {
            wield_writer.send(ActorEvent::new(actor_entity, Wield { item }));
        }
        PlannedAction::Unwield { item } => {
            unwield_writer.send(ActorEvent::new(actor_entity, Unwield { item }));
        }
        PlannedAction::Pickup { item } => {
            pickup_writer.send(ActorEvent::new(actor_entity, Pickup { item }));
        }
        PlannedAction::MoveItem { item, to } => {
            move_item_writer.send(ActorEvent::new(actor_entity, MoveItem { item, to }));
        }
        PlannedAction::ExamineItem { item } => {
            examine_item_writer.send(ActorEvent::new(actor_entity, ExamineItem { item }));
        }
        PlannedAction::ChangePace => {
            change_pace_writer.send(ActorEvent::new(actor_entity, ChangePace));
        }
    };

    log_if_slow("send_action_event", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn check_action_plan_amount(
    mut stay_reader: EventReader<ActorEvent<Stay>>,
    mut step_reader: EventReader<ActorEvent<Step>>,
    mut attack_reader: EventReader<ActorEvent<Attack>>,
    mut smash_reader: EventReader<ActorEvent<Smash>>,
    mut close_reader: EventReader<ActorEvent<Close>>,
    mut wield_reader: EventReader<ActorEvent<Wield>>,
    mut unwield_reader: EventReader<ActorEvent<Unwield>>,
    mut pickup_reader: EventReader<ActorEvent<Pickup>>,
    mut move_item_reader: EventReader<ActorEvent<MoveItem>>,
    mut examine_item_reader: EventReader<ActorEvent<ExamineItem>>,
    mut change_pace_reader: EventReader<ActorEvent<ChangePace>>,
) {
    let all = stay_reader
        .read()
        .map(|a| format!("{a:?}"))
        .chain(step_reader.read().map(|a| format!("{a:?}")))
        .chain(attack_reader.read().map(|a| format!("{a:?}")))
        .chain(smash_reader.read().map(|a| format!("{a:?}")))
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
pub(crate) fn single_action<T: ActorChange>(
    mut action_reader: EventReader<ActorEvent<T>>,
) -> ActorEvent<T> {
    action_reader.read().next().cloned().expect("Single event")
}

#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::unnecessary_wraps)]
pub(crate) fn perform_stay(In(stay): In<ActorEvent<Stay>>, actors: Query<Actor>) -> Option<Impact> {
    Some(stay.actor(&actors).stay(&stay.change))
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_step(
    In(step): In<ActorEvent<Step>>,
    mut commands: Commands,
    mut message_writer: EventWriter<Message>,
    mut toggle_writer: EventWriter<TerrainEvent<Toggle>>,
    mut envir: Envir,
    actors: Query<Actor>,
) -> Option<Impact> {
    step.actor(&actors).step(
        &mut commands,
        &mut message_writer,
        &mut toggle_writer,
        &mut envir,
        &step.change,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_attack(
    In(attack): In<ActorEvent<Attack>>,
    mut message_writer: EventWriter<Message>,
    mut damage_writer: EventWriter<ActorEvent<Damage>>,
    envir: Envir,
    infos: Res<Infos>,
    hierarchy: Hierarchy,
    actors: Query<Actor>,
) -> Option<Impact> {
    attack.actor(&actors).attack(
        &mut message_writer,
        &mut damage_writer,
        &envir,
        &infos,
        &hierarchy,
        &attack.change,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_smash(
    In(smash): In<ActorEvent<Smash>>,
    mut message_writer: EventWriter<Message>,
    mut damage_writer: EventWriter<TerrainEvent<Damage>>,
    envir: Envir,
    infos: Res<Infos>,
    hierarchy: Hierarchy,
    actors: Query<Actor>,
) -> Option<Impact> {
    smash.actor(&actors).smash(
        &mut message_writer,
        &mut damage_writer,
        &envir,
        &infos,
        &hierarchy,
        &smash.change,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_close(
    In(close): In<ActorEvent<Close>>,
    mut message_writer: EventWriter<Message>,
    mut toggle_writer: EventWriter<TerrainEvent<Toggle>>,
    envir: Envir,
    actors: Query<Actor>,
) -> Option<Impact> {
    close.actor(&actors).close(
        &mut message_writer,
        &mut toggle_writer,
        &envir,
        &close.change,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_wield(
    In(wield): In<ActorEvent<Wield>>,
    mut commands: Commands,
    mut message_writer: EventWriter<Message>,
    mut location: ResMut<Location>,
    hierarchy: Hierarchy,
    actors: Query<Actor>,
) -> Option<Impact> {
    wield.actor(&actors).wield(
        &mut commands,
        &mut message_writer,
        &mut location,
        &hierarchy,
        &wield.change,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_unwield(
    In(unwield): In<ActorEvent<Unwield>>,
    mut commands: Commands,
    mut message_writer: EventWriter<Message>,
    mut location: ResMut<Location>,
    hierarchy: Hierarchy,
    actors: Query<Actor>,
) -> Option<Impact> {
    unwield.actor(&actors).unwield(
        &mut commands,
        &mut message_writer,
        &mut location,
        &hierarchy,
        &unwield.change,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_pickup(
    In(pickup): In<ActorEvent<Pickup>>,
    mut commands: Commands,
    mut message_writer: EventWriter<Message>,
    mut location: ResMut<Location>,
    hierarchy: Hierarchy,
    actors: Query<Actor>,
) -> Option<Impact> {
    pickup.actor(&actors).pickup(
        &mut commands,
        &mut message_writer,
        &mut location,
        &hierarchy,
        &pickup.change,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_move_item(
    In(move_item): In<ActorEvent<MoveItem>>,
    mut commands: Commands,
    mut message_writer: EventWriter<Message>,
    subzone_level_entities: Res<SubzoneLevelEntities>,
    mut location: ResMut<Location>,
    hierarchy: Hierarchy,
    actors: Query<Actor>,
) -> Option<Impact> {
    move_item.actor(&actors).move_item(
        &mut commands,
        &mut message_writer,
        &subzone_level_entities,
        &mut location,
        &hierarchy,
        &move_item.change,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_examine_item(
    In(examine_item): In<ActorEvent<ExamineItem>>,
    mut message_writer: EventWriter<Message>,
    infos: Res<Infos>,
    definitions: Query<&ObjectDefinition>,
) {
    ActorItem::examine_item(
        &mut message_writer,
        &infos,
        &definitions,
        &examine_item.change,
    );
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn perform_change_pace(
    In(change_pace): In<ActorEvent<ChangePace>>,
    mut commands: Commands,
    actors: Query<Actor>,
) {
    change_pace.actor(&actors).change_pace(&mut commands);
}

pub(crate) fn proces_impact(
    In(impact): In<Option<Impact>>,
    mut stamina_impact_events: EventWriter<ActorEvent<StaminaImpact>>,
    mut timeout_events: EventWriter<ActorEvent<Timeout>>,
) {
    let Some(impact) = impact else {
        return;
    };

    stamina_impact_events.send(ActorEvent::new(impact.actor_entity, impact.stamina_impact));
    timeout_events.send(ActorEvent::new(
        impact.actor_entity,
        Timeout {
            delay: impact.timeout,
        },
    ));
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_timeout(
    mut timeout_events: EventReader<ActorEvent<Timeout>>,
    mut message_writer: EventWriter<Message>,
    mut timeouts: ResMut<Timeouts>,
    players: Query<Entity, With<Player>>,
) {
    let start = Instant::now();

    match timeout_events.len() {
        0 => {
            // None when waiting for player input

            if let Some(actor_entity) = timeouts.next(&players.iter().collect::<Vec<_>>()) {
                if players.get(actor_entity).is_err() {
                    message_writer.send(Message::error(Phrase::new("NPC action failed")));
                    timeouts.add(actor_entity, Milliseconds(1000));
                }
            }
        }
        1 => {
            for timeout_event in timeout_events.read() {
                assert!(0 < timeout_event.change.delay.0, "{timeout_event:?}");
                timeouts.add(timeout_event.actor_entity, timeout_event.change.delay);
            }
        }
        _ => {
            panic!("Multiple timeout events: {timeout_events:?}");
        }
    }

    log_if_slow("update_timeout", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_stamina(
    mut timeout_events: EventReader<ActorEvent<Timeout>>,
    mut stamina_impact_events: EventReader<ActorEvent<StaminaImpact>>,
    mut staminas: Query<&mut Stamina>,
) {
    let start = Instant::now();

    assert!(
        stamina_impact_events.len() <= 1,
        "Multiple stamina impact events: {:?}",
        stamina_impact_events.read().collect::<Vec<_>>()
    );

    assert!(
        stamina_impact_events.len() <= timeout_events.len(),
        "More stamina impact events than timeout events: {:?} {:?}",
        timeout_events.read().collect::<Vec<_>>(),
        stamina_impact_events.read().collect::<Vec<_>>()
    );

    for stamina_impact_event in stamina_impact_events.read() {
        if let Ok(mut stamina) = staminas.get_mut(stamina_impact_event.actor_entity) {
            stamina.apply(stamina_impact_event.change);
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
    mut damage_reader: EventReader<ActorEvent<Damage>>,
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
        let evolution = health.lower(&damage.change);
        if health.0.is_zero() {
            message_writer.send(Message::warn(
                damage
                    .change
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
                .insert(ObjectName::corpse())
                .remove::<Life>()
                .remove::<Obstacle>();

            if player.is_some() {
                next_gameplay_state.set(GameplayScreenState::Death);
            }
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

    log_if_slow("update_damaged_characters", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_healed_characters(
    mut message_writer: EventWriter<Message>,
    mut healing_reader: EventReader<ActorEvent<Healing>>,
    mut actors: ParamSet<(
        Query<&mut Health, (With<Faction>, With<Life>)>,
        Query<Actor, (With<Faction>, With<Life>)>,
    )>,
) {
    let start = Instant::now();

    for healing in healing_reader.read() {
        let mut healths = actors.p0();
        let mut health = healths.get_mut(healing.actor_entity).expect("Actor found");
        let evolution = health.raise(&healing.change);
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
    moved_items: Query<
        Item,
        (
            Changed<Parent>,
            Without<Container>, /*, With<Container>*/
        ), //TODO this takes very long for initial entities
    >,
) {
    let start = Instant::now();

    let mut all_merged = Vec::new();

    for moved in &moved_items {
        if moved.definition.category == ObjectCategory::Item && !all_merged.contains(&moved.entity)
        {
            let mut merges = vec![moved.entity];
            let mut total_amount = &Amount(0) + moved.amount;

            if let Some(moved_parent) = moved.parent {
                for sibling in hierarchy.items_in(moved_parent.get()) {
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
