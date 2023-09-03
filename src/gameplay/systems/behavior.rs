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
    mut player_action_state: ResMut<PlayerActionState>,
    mut envir: Envir,
    clock: Clock,
    mut instruction_queue: ResMut<InstructionQueue>,
    actors: Query<Actor>,
    factions: Query<(&Pos, &Faction), With<Health>>,
    mut players: Query<(), With<Player>>,
) -> Option<(Entity, Action)> {
    let start = Instant::now();

    let Some(active_entity) = option else {
        eprintln!("No egible characters!");
        return None;
    };

    let factions = &factions.iter().map(|(p, f)| (*p, f)).collect::<Vec<_>>();
    let actor = actors.get(active_entity).unwrap();
    let enemies = Faction::Human.enemies(&envir, &clock, factions, &actor);
    let action = if players.get_mut(active_entity).is_ok() {
        player_action_state.plan_action(
            &mut commands,
            &mut envir,
            &mut instruction_queue,
            actor.entity,
            *actor.pos,
            clock.time(),
            &enemies,
        )?
    } else {
        let strategy = actor.faction.strategize(&envir, &clock, factions, &actor);
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

    let Some((actor_entity, action)) = option  else {
        return None;
    };

    let impact = match action {
        Action::Stay { duration } => {
            perform_with_actor::<(), _>(world, actor_entity, |actor, ()| Some(actor.stay(duration)))
        }
        Action::Step { target } => perform_with_actor::<(Commands, Envir), _>(
            world,
            actor_entity,
            |actor, (mut commands, mut envir)| actor.move_(&mut commands, &mut envir, target),
        ),
        Action::Attack { target } => {
            perform_with_actor::<(Commands, Envir, Res<Infos>, Hierarchy), _>(
                world,
                actor_entity,
                |actor, (mut commands, envir, infos, hierarchy)| {
                    actor.attack(&mut commands, &envir, &infos, &hierarchy, target)
                },
            )
        }
        Action::Smash { target } => {
            perform_with_actor::<(Commands, Envir, Res<Infos>, Hierarchy), _>(
                world,
                actor_entity,
                |actor, (mut commands, envir, infos, hierarchy)| {
                    actor.smash(&mut commands, &envir, &infos, &hierarchy, target)
                },
            )
        }
        Action::Close { target } => perform_with_actor::<(Commands, Envir), _>(
            world,
            actor_entity,
            |actor, (mut commands, mut envir)| actor.close(&mut commands, &mut envir, target),
        ),
        Action::Wield { entity } => {
            perform_with_actor::<(Commands, ResMut<Location>, Hierarchy), _>(
                world,
                actor_entity,
                |actor, (mut commands, mut location, hierarchy)| {
                    actor.wield(&mut commands, &mut location, &hierarchy, entity)
                },
            )
        }
        Action::Unwield { entity } => {
            perform_with_actor::<(Commands, ResMut<Location>, Hierarchy), _>(
                world,
                actor_entity,
                |actor, (mut commands, mut location, hierarchy)| {
                    actor.unwield(&mut commands, &mut location, &hierarchy, entity)
                },
            )
        }
        Action::Pickup { entity } => {
            perform_with_actor::<(Commands, ResMut<Location>, Hierarchy), _>(
                world,
                actor_entity,
                |actor, (mut commands, mut location, hierarchy)| {
                    actor.pickup(&mut commands, &mut location, &hierarchy, entity)
                },
            )
        }
        Action::Dump { entity, direction } => {
            perform_with_actor::<(Commands, ResMut<Location>, Hierarchy), _>(
                world,
                actor_entity,
                |actor, (mut commands, mut location, hierarchy)| {
                    actor.dump(&mut commands, &mut location, &hierarchy, entity, direction)
                },
            )
        }
        Action::ExamineItem { entity } => perform::<
            (Commands, Res<Infos>, Query<&ObjectDefinition>),
            _,
        >(world, |(mut commands, infos, definitions)| {
            ActorItem::examine_item(&mut commands, &infos, &definitions, entity);
            None
        }),
        Action::SwitchRunning => {
            perform_with_actor::<Commands, _>(world, actor_entity, |actor, mut commands| {
                actor.switch_running(&mut commands);
                None
            })
        }
    };

    log_if_slow("manage_characters", start);

    Some((actor_entity, impact))
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
    mut commands: Commands,
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
            commands.spawn(Message::error(Phrase::new("NPC action failed")));
            timeouts.add(actor_entity, Milliseconds(1000));
        }
    }

    log_if_slow("handle_impact", start);
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn manage_player_death(
    mut next_application_state: ResMut<NextState<ApplicationState>>,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    dead_players: Query<(), (With<Player>, Without<Health>)>,
) {
    let start = Instant::now();

    if dead_players.get_single().is_ok() {
        next_gameplay_state.set(GameplayScreenState::Inapplicable);
        next_application_state.set(ApplicationState::MainMenu);
    }

    log_if_slow("manage_player_death", start);
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
    mut characters: Query<
        (Entity, &ObjectName, &mut Health, &Damage, &mut Transform),
        With<Faction>,
    >,
) {
    let start = Instant::now();

    for (character, name, mut health, damage, mut transform) in &mut characters {
        let evolution = health.lower(damage);
        if health.0.is_zero() {
            commands.spawn(Message::warn(
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
                .remove::<Health>()
                .remove::<Obstacle>();
        } else {
            commands.spawn({
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
    mut characters: Query<
        (Entity, &ObjectName, &mut Health, &Healing, &mut Transform),
        With<Faction>,
    >,
) {
    let start = Instant::now();

    for (character, name, mut health, healing, _transform) in &mut characters {
        let evolution = health.raise(healing);
        if evolution.changed() {
            commands.spawn({
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
            commands.spawn(Message::warn(
                Phrase::from_fragment(damage.attacker.clone())
                    .add("breaks")
                    .extend(name.as_item(amount, filthy)),
            ));
            commands.entity(item).despawn_recursive();
            spawner.spawn_smashed(&infos, parent.get(), pos, definition);
            *visualization_update = VisualizationUpdate::Forced;
        } else {
            commands.spawn({
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
