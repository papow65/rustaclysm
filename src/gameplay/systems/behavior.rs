use crate::prelude::*;
use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
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
    In(option): In<Option<Entity>>,
    mut commands: Commands,
    mut message_writer: EventWriter<Message>,
    mut player_action_state: ResMut<PlayerActionState>,
    mut envir: Envir,
    clock: Clock,
    mut instruction_queue: ResMut<InstructionQueue>,
    actors: Query<Actor>,
    factions: Query<(&Pos, &Faction), With<Life>>,
    mut players: Query<(), With<Player>>,
) -> Option<(Entity, Action)> {
    let start = Instant::now();

    let Some(active_entity) = option else {
        eprintln!("No egible characters!");
        return None;
    };

    let factions = &factions.iter().map(|(p, f)| (*p, f)).collect::<Vec<_>>();
    let actor = actors.get(active_entity).unwrap();
    let enemies = actor.faction.enemies(&envir, &clock, factions, &actor);
    let action = if players.get_mut(active_entity).is_ok() {
        player_action_state.plan_action(
            &mut commands,
            &mut message_writer,
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
pub(crate) fn perform_action(
    In(option): In<Option<(Entity, Action)>>,
    world: &mut World,
) -> Option<(Entity, Option<Impact>)> {
    let start = Instant::now();

    let Some((actor_entity, action)) = option else {
        return None;
    };

    let impact = match action {
        Action::Stay { duration } => perform_stay(world, actor_entity, duration),
        Action::Step { target } => perform_stap(world, actor_entity, target),
        Action::Attack { target } => perform_attack(world, actor_entity, target),
        Action::Smash { target } => perform_smash(world, actor_entity, target),
        Action::Close { target } => perform_close(world, actor_entity, target),
        Action::Wield { entity } => perform_wield(world, actor_entity, entity),
        Action::Unwield { entity } => perform_unwield(world, actor_entity, entity),
        Action::Pickup { entity } => perform_pickup(world, actor_entity, entity),
        Action::Dump { entity, direction } => perform_dump(world, actor_entity, entity, direction),
        Action::ExamineItem { entity } => perform_examine_item(world, entity),
        Action::SwitchRunning => perform_switch_running(world, actor_entity),
    };

    log_if_slow("perform_action", start);

    Some((actor_entity, impact))
}

fn perform_stay(world: &mut World, actor_entity: Entity, duration: StayDuration) -> Option<Impact> {
    perform_with_actor::<(), _>(world, actor_entity, |actor, ()| Some(actor.stay(duration)))
}

fn perform_stap(world: &mut World, actor_entity: Entity, target: Pos) -> Option<Impact> {
    perform_with_actor::<(Commands, EventWriter<Message>, Envir), _>(
        world,
        actor_entity,
        |actor, (mut commands, mut message_writer, mut envir)| {
            actor.move_(&mut commands, &mut message_writer, &mut envir, target)
        },
    )
}

fn perform_attack(world: &mut World, actor_entity: Entity, target: Pos) -> Option<Impact> {
    perform_with_actor::<(Commands, EventWriter<Message>, Envir, Res<Infos>, Hierarchy), _>(
        world,
        actor_entity,
        |actor, (mut commands, mut message_writer, envir, infos, hierarchy)| {
            actor.attack(
                &mut commands,
                &mut message_writer,
                &envir,
                &infos,
                &hierarchy,
                target,
            )
        },
    )
}

fn perform_smash(world: &mut World, actor_entity: Entity, target: Pos) -> Option<Impact> {
    perform_with_actor::<(Commands, EventWriter<Message>, Envir, Res<Infos>, Hierarchy), _>(
        world,
        actor_entity,
        |actor, (mut commands, mut message_writer, envir, infos, hierarchy)| {
            actor.smash(
                &mut commands,
                &mut message_writer,
                &envir,
                &infos,
                &hierarchy,
                target,
            )
        },
    )
}

fn perform_close(world: &mut World, actor_entity: Entity, target: Pos) -> Option<Impact> {
    perform_with_actor::<(Commands, EventWriter<Message>, Envir), _>(
        world,
        actor_entity,
        |actor, (mut commands, mut message_writer, mut envir)| {
            actor.close(&mut commands, &mut message_writer, &mut envir, target)
        },
    )
}

fn perform_wield(world: &mut World, actor_entity: Entity, entity: Entity) -> Option<Impact> {
    perform_with_actor::<(Commands, EventWriter<Message>, ResMut<Location>, Hierarchy), _>(
        world,
        actor_entity,
        |actor, (mut commands, mut message_writer, mut location, hierarchy)| {
            actor.wield(
                &mut commands,
                &mut message_writer,
                &mut location,
                &hierarchy,
                entity,
            )
        },
    )
}

fn perform_unwield(world: &mut World, actor_entity: Entity, entity: Entity) -> Option<Impact> {
    perform_with_actor::<(Commands, EventWriter<Message>, ResMut<Location>, Hierarchy), _>(
        world,
        actor_entity,
        |actor, (mut commands, mut message_writer, mut location, hierarchy)| {
            actor.unwield(
                &mut commands,
                &mut message_writer,
                &mut location,
                &hierarchy,
                entity,
            )
        },
    )
}

fn perform_pickup(world: &mut World, actor_entity: Entity, entity: Entity) -> Option<Impact> {
    perform_with_actor::<(Commands, EventWriter<Message>, ResMut<Location>, Hierarchy), _>(
        world,
        actor_entity,
        |actor, (mut commands, mut message_writer, mut location, hierarchy)| {
            actor.pickup(
                &mut commands,
                &mut message_writer,
                &mut location,
                &hierarchy,
                entity,
            )
        },
    )
}

fn perform_dump(
    world: &mut World,
    actor_entity: Entity,
    entity: Entity,
    direction: HorizontalDirection,
) -> Option<Impact> {
    perform_with_actor::<(Commands, EventWriter<Message>, ResMut<Location>, Hierarchy), _>(
        world,
        actor_entity,
        |actor, (mut commands, mut message_writer, mut location, hierarchy)| {
            actor.dump(
                &mut commands,
                &mut message_writer,
                &mut location,
                &hierarchy,
                entity,
                direction,
            )
        },
    )
}

fn perform_examine_item(world: &mut World, entity: Entity) -> Option<Impact> {
    perform::<(EventWriter<Message>, Res<Infos>, Query<&ObjectDefinition>), _>(
        world,
        |(mut message_writer, infos, definitions)| {
            ActorItem::examine_item(&mut message_writer, &infos, &definitions, entity);
            None
        },
    )
}

fn perform_switch_running(world: &mut World, actor_entity: Entity) -> Option<Impact> {
    perform_with_actor::<Commands, _>(world, actor_entity, |actor, mut commands| {
        actor.switch_running(&mut commands);
        None
    })
}

fn perform_with_actor<P, F>(world: &mut World, actor_entity: Entity, act: F) -> Option<Impact>
where
    P: SystemParam + 'static,
    for<'a, 'b> F: Fn(ActorItem, <P as SystemParam>::Item<'a, 'b>) -> Option<Impact>,
{
    perform::<(Query<Actor>, P), _>(world, |(actors, p)| {
        let actor = actors.get(actor_entity).unwrap();
        act(actor, p)
    })
}

fn perform<P, F>(world: &mut World, act: F) -> Option<Impact>
where
    P: SystemParam + 'static,
    for<'a, 'b> F: Fn(<P as SystemParam>::Item<'a, 'b>) -> Option<Impact>,
{
    let mut system_state = SystemState::<P>::new(world);
    let p = system_state.get_mut(world);
    let impact = act(p);
    system_state.apply(world);
    impact
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn handle_impact(
    In(option): In<Option<(Entity, Option<Impact>)>>,
    mut message_writer: EventWriter<Message>,
    mut timeouts: ResMut<Timeouts>,
    players: Query<(), With<Player>>,
    mut staminas: Query<&mut Stamina>,
) {
    let start = Instant::now();

    // None when waiting for player input
    if let Some((actor_entity, impact)) = option {
        if let Some(impact) = impact {
            if let Ok(mut stamina) = staminas.get_mut(actor_entity) {
                stamina.apply(impact.stamina_impact);
            }
            assert!(0 < impact.timeout.0, "{impact:?}");
            timeouts.add(actor_entity, impact.timeout);
        } else if players.get(actor_entity).is_err() {
            message_writer.send(Message::error(Phrase::new("NPC action failed")));
            timeouts.add(actor_entity, Milliseconds(1000));
        }
    }

    log_if_slow("handle_impact", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn toggle_doors(
    mut commands: Commands,
    infos: Res<Infos>,
    mut spawner: Spawner,
    mut visualization_update: ResMut<VisualizationUpdate>,
    toggled: Query<(Entity, &ObjectDefinition, &Pos, &Toggle, &Parent)>,
) {
    let start = Instant::now();

    for (entity, definition, &pos, toggle, parent) in toggled.iter() {
        commands.entity(entity).despawn_recursive();
        let terrain_info = infos.terrain(&definition.id).expect("Valid terrain");
        let toggled_id = match toggle {
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
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    mut characters: Query<
        (
            Entity,
            &ObjectName,
            &mut Health,
            &Damage,
            &mut Transform,
            Option<&Player>,
        ),
        (With<Faction>, With<Life>),
    >,
) {
    let start = Instant::now();

    for (character, name, mut health, damage, mut transform, player) in &mut characters {
        let evolution = health.lower(damage);
        if health.0.is_zero() {
            message_writer.send(Message::warn(
                Phrase::from_fragment(damage.attacker.clone())
                    .add("kills")
                    .push(name.single()),
            ));
            transform.rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)
                * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
            commands
                .entity(character)
                .insert(Corpse)
                .insert(ObjectName::corpse())
                .remove::<Life>()
                .remove::<Obstacle>();

            if player.is_some() {
                next_gameplay_state.set(GameplayScreenState::Death);
            }
        } else {
            message_writer.send({
                let begin = Phrase::from_fragment(damage.attacker.clone())
                    .add("hits")
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

        commands.entity(character).remove::<Damage>();
    }

    log_if_slow("update_damaged_characters", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_healed_characters(
    mut commands: Commands,
    mut message_writer: EventWriter<Message>,
    mut characters: Query<
        (Entity, &ObjectName, &mut Health, &Healing, &mut Transform),
        (With<Faction>, With<Life>),
    >,
) {
    let start = Instant::now();

    for (character, name, mut health, healing, _transform) in &mut characters {
        let evolution = health.raise(healing);
        if evolution.changed() {
            message_writer.send({
                let begin = Phrase::from_name(name);

                Message::info(if evolution.change_abs() == 1 {
                    begin.add("heals a bit")
                } else {
                    begin.add(format!(
                        "heals for {} ({} -> {})",
                        evolution.change_abs(),
                        evolution.before,
                        evolution.after
                    ))
                })
            });
        }
        commands.entity(character).remove::<Healing>();
    }

    log_if_slow("update_healed_characters", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn update_damaged_items(
    mut commands: Commands,
    mut message_writer: EventWriter<Message>,
    mut spawner: Spawner,
    mut visualization_update: ResMut<VisualizationUpdate>,
    infos: Res<Infos>,
    mut damaged: Query<(
        Entity,
        &Pos,
        &ObjectName,
        Option<&Amount>,
        Option<&Filthy>,
        &mut Integrity,
        &Damage,
        &ObjectDefinition,
        &Parent,
    )>,
) {
    let start = Instant::now();

    for (item, &pos, name, amount, filthy, mut integrity, damage, definition, parent) in
        &mut damaged
    {
        let evolution = integrity.lower(damage);
        if integrity.0.is_zero() {
            message_writer.send(Message::warn(
                Phrase::from_fragment(damage.attacker.clone())
                    .add("breaks")
                    .extend(name.as_item(amount, filthy)),
            ));
            commands.entity(item).despawn_recursive();
            spawner.spawn_smashed(&infos, parent.get(), pos, definition);
            *visualization_update = VisualizationUpdate::Forced;
        } else {
            message_writer.send({
                let begin = Phrase::from_fragment(damage.attacker.clone())
                    .add("hits")
                    .extend(name.as_item(amount, filthy));

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
        (
            Entity,
            &ObjectDefinition,
            &ObjectName,
            Option<&Pos>,
            Option<&Amount>,
            Option<&Filthy>,
            &Parent,
        ),
        (
            Changed<Parent>,
            Without<Container>, /*, With<Container>*/
        ), //TODO this takes very long for initial entities
    >,
) {
    let start = Instant::now();

    let mut all_merged = Vec::new();

    for (
        moved_entity,
        moved_definition,
        moved_name,
        moved_pos,
        moved_amount,
        moved_filthy,
        moved_parent,
    ) in moved_items.iter()
    {
        if moved_definition.category == ObjectCategory::Item && !all_merged.contains(&moved_entity)
        {
            let (_, siblings) = hierarchy
                .parents
                .get(moved_parent.get())
                .unwrap_or_else(|_| {
                    //TODO fix this panic after moving items around

                    panic!(
                        "Parent of {} could not be found",
                        Phrase::from_fragments(moved_name.as_item(moved_amount, moved_filthy))
                    )
                });

            let mut merges = vec![moved_entity];
            let mut total_amount = &Amount(0) + moved_amount.unwrap_or(&Amount(1));

            for sibling in siblings {
                // Note that sibling may be any kind of entity
                if let Ok((
                    some_entity,
                    some_definition,
                    _,
                    some_pos,
                    some_amount,
                    some_filthy,
                    _,
                    _,
                )) = hierarchy.items.get(*sibling)
                {
                    // Note that the positions may differ when the parents are the same.
                    if some_entity != moved_entity
                        && moved_definition == some_definition
                        && moved_pos == some_pos
                        && moved_filthy == some_filthy
                        && !all_merged.contains(&some_entity)
                    {
                        merges.push(some_entity);
                        total_amount = &total_amount + some_amount.unwrap_or(&Amount(1));
                        all_merged.push(some_entity);
                    }
                }
            }

            if 1 < merges.len() {
                let keep = *merges.iter().max().unwrap();

                println!(
                    "Combined {} with {} others to {}",
                    Phrase::from_fragments(moved_name.as_item(moved_amount, moved_filthy)),
                    merges.len() - 1,
                    Phrase::from_fragments(moved_name.as_item(Some(&total_amount), moved_filthy)),
                );

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
