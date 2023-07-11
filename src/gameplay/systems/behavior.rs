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
    actors: Actors,
    players: Query<(), With<Player>>,
) -> Option<Entity> {
    let egible_entities = actors
        .actors()
        .filter(|a| envir.is_accessible(a.pos) || players.get(a.entity).is_ok())
        .map(|a| a.entity)
        .collect::<Vec<Entity>>();
    timeouts.next(&egible_entities)
}

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn plan_action(
    In(option): In<Option<Entity>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameplayScreenState>>,
    mut player_action_state: ResMut<PlayerActionState>,
    mut envir: Envir,
    clock: Clock,
    mut instruction_queue: ResMut<InstructionQueue>,
    actors: Actors,
    mut players: Query<(), With<Player>>,
) -> Option<(Entity, Action)> {
    let start = Instant::now();

    let Some(active_entity) = option else {
        eprintln!("No egible characters!");
        return None;
    };

    let factions = actors.collect_factions();
    let actor = actors.get(active_entity);
    let enemies = Faction::Human.enemies(&envir, &clock, &factions, &actor);
    let action = if players.get_mut(active_entity).is_ok() {
        player_action_state.plan_action(
            &mut commands,
            &mut next_state,
            &mut envir,
            &mut instruction_queue,
            actor.pos,
            clock.time(),
            &enemies,
        )?
    } else {
        let strategy = actor.faction.strategize(&envir, &clock, &factions, &actor);
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
        Action::Dump { entity } => {
            perform_with_actor::<(Commands, ResMut<Location>, Hierarchy), _>(
                world,
                actor_entity,
                |actor, (mut commands, mut location, hierarchy)| {
                    Some(actor.dump(&mut commands, &mut location, &hierarchy, entity))
                },
            )
        }
        Action::ExamineItem { entity } => perform::<
            (Commands, Res<Infos>, Query<&ObjectDefinition>),
            _,
        >(world, |(mut commands, infos, definitions)| {
            Actor::examine_item(&mut commands, &infos, &definitions, entity);
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
    for<'a, 'b> F: Fn(Actor, <P as SystemParam>::Item<'a, 'b>) -> Option<Impact>,
{
    perform::<(Actors, P), _>(world, |(actors, p)| act(actors.get(actor_entity), p))
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
    _healths: Query<&mut Health>,
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
            commands.spawn(Message::error().str("failed npc action"));
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

    for (character, name, mut health, damage, mut transform) in characters.iter_mut() {
        let evolution = health.lower(damage);
        if health.0.is_zero() {
            commands.spawn(
                Message::warn()
                    .push(damage.attacker.clone())
                    .str("kills")
                    .push(name.single()),
            );
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
                let begin = Message::warn()
                    .push(damage.attacker.clone())
                    .str("hits")
                    .push(name.single());

                if evolution.changed() {
                    begin.add(format!(
                        "for {} ({} -> {})",
                        evolution.before - evolution.after,
                        evolution.before,
                        evolution.after
                    ))
                } else {
                    begin.add(String::from("but it has no effect"))
                }
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

    for (character, name, mut health, healing, _transform) in characters.iter_mut() {
        let evolution = health.raise(healing);
        if evolution.changed() {
            commands.spawn(Message::warn().push(name.single()).add(format!(
                "heals for {} ({} -> {})",
                evolution.after - evolution.before,
                evolution.before,
                evolution.after
            )));
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
        damaged.iter_mut()
    {
        let evolution = integrity.lower(damage);
        if integrity.0.is_zero() {
            commands.spawn(
                Message::warn()
                    .push(damage.attacker.clone())
                    .str("breaks")
                    .extend(name.as_item(amount, filthy)),
            );
            commands.entity(item).despawn_recursive();
            spawner.spawn_smashed(&infos, parent.get(), pos, definition);
            *visualization_update = VisualizationUpdate::Forced;
        } else {
            commands.spawn({
                let begin = Message::warn()
                    .push(damage.attacker.clone())
                    .str("hits")
                    .extend(name.as_item(amount, filthy));

                if evolution.changed() {
                    begin.add(format!(
                        "for {} ({} -> {})",
                        evolution.before - evolution.after,
                        evolution.before,
                        evolution.after
                    ))
                } else {
                    begin.add(String::from("but it has no effect"))
                }
            });
        }
    }

    log_if_slow("update_damaged_items", start);
}
