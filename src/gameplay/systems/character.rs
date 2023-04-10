use crate::prelude::*;
use bevy::{ecs::system::SystemState, prelude::*};
use std::time::Instant;

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
pub(crate) fn plan_action(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameplayScreenState>>,
    mut player_action_state: ResMut<PlayerActionState>,
    mut envir: Envir,
    mut instruction_queue: ResMut<InstructionQueue>,
    mut timeouts: ResMut<Timeouts>,
    actors: Actors,
    mut players: Query<(), With<Player>>,
) -> Option<(Entity, Action)> {
    let start = Instant::now();

    let egible_entities = actors
        .actors()
        .filter(|a| envir.is_accessible(a.pos) || players.get(a.entity).is_ok())
        .map(|a| a.entity)
        .collect::<Vec<Entity>>();
    let Some(active_entity) = timeouts.next(&egible_entities) else {
        eprintln!("No egible characters!");
        return None;
    };
    let factions = actors.collect_factions();
    let actor = actors.get(active_entity);
    let action = if players.get_mut(active_entity).is_ok() {
        if let Some(action) = player_action_state.plan_action(
            &mut commands,
            &mut next_state,
            &mut envir,
            &mut instruction_queue,
            actor.pos,
            timeouts.time(),
        ) {
            action
        } else if let PlayerActionState::Waiting(until) = *player_action_state {
            if !Faction::Human.enemies(&envir, &factions, &actor).is_empty() {
                instruction_queue.add(QueuedInstruction::Interrupted);
                return None; // process the cancellation next turn
            } else if until <= timeouts.time() {
                instruction_queue.add(QueuedInstruction::Finished);
                return None; // process the cancellation next turn
            } else {
                Action::Stay
            }
        } else {
            instruction_queue.start_waiting();
            println!("Waiting for user action");
            return None; // no key pressed - wait for the user
        }
    } else {
        let strategy = actor.faction.strategize(&envir, &factions, &actor);
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
        Action::Stay => perform(world, actor_entity, |actor| Some(actor.stay())),
        Action::Step { target } => {
            perform_commands_envir(world, actor_entity, |actor, commands, envir| {
                actor.move_(commands, envir, target)
            })
        }
        Action::Attack { target } => {
            perform_commands_envir(world, actor_entity, |actor, commands, envir| {
                actor.attack(commands, envir, target)
            })
        }
        Action::Smash { target } => {
            perform_commands_envir(world, actor_entity, |actor, commands, envir| {
                actor.smash(commands, envir, target)
            })
        }
        Action::Close { target } => {
            perform_commands_envir(world, actor_entity, |actor, commands, envir| {
                actor.close(commands, envir, target)
            })
        }
        Action::Wield => perform_commands_location_hierarchy(
            world,
            actor_entity,
            |actor, commands, location, hierarchy| actor.wield(commands, location, hierarchy),
        ),
        Action::Pickup => perform_commands_location_hierarchy(
            world,
            actor_entity,
            |actor, commands, location, hierarchy| actor.pickup(commands, location, hierarchy),
        ),
        Action::Dump => {
            let mut system_state = SystemState::<(
                Commands,
                Envir,
                Query<(Entity, &ObjectName, &Amount, Option<&Filthy>, &Parent)>,
                Actors,
            )>::new(world);
            let (mut commands, mut envir, dumpees, actors) = system_state.get_mut(world);
            let impact =
                actors
                    .get(actor_entity)
                    .dump(&mut commands, &mut envir.location, &dumpees);
            system_state.apply(world);
            impact
        }
        Action::SwitchRunning => {
            perform_commands_envir(world, actor_entity, |actor, commands, _| {
                actor.switch_running(commands)
            })
        }
    };

    log_if_slow("manage_characters", start);

    Some((actor_entity, impact))
}

// TODO combine all versions of 'perform' in a generic function

fn perform<F>(world: &mut World, actor_entity: Entity, act: F) -> Option<Impact>
where
    F: Fn(Actor) -> Option<Impact>,
{
    let mut system_state = SystemState::<(Actors,)>::new(world);
    let (actors,) = system_state.get_mut(world);
    let impact = act(actors.get(actor_entity));
    system_state.apply(world);
    impact
}

fn perform_commands_envir<F>(world: &mut World, actor_entity: Entity, act: F) -> Option<Impact>
where
    F: Fn(Actor, &mut Commands, &mut Envir) -> Option<Impact>,
{
    let mut system_state = SystemState::<(Commands, Envir, Actors)>::new(world);
    let (mut commands, mut envir, actors) = system_state.get_mut(world);
    let impact = act(actors.get(actor_entity), &mut commands, &mut envir);
    system_state.apply(world);
    impact
}

fn perform_commands_location_hierarchy<F>(
    world: &mut World,
    actor_entity: Entity,
    act: F,
) -> Option<Impact>
where
    F: Fn(Actor, &mut Commands, &mut Location, &Hierarchy) -> Option<Impact>,
{
    let mut system_state =
        SystemState::<(Commands, Envir, Hierarchy, Actors<'static, 'static>)>::new(world);
    let (mut commands, mut envir, hierarchy, actors) = system_state.get_mut(world);
    let impact = act(
        actors.get(actor_entity),
        &mut commands,
        &mut envir.location,
        &hierarchy,
    );
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
        let stamina = staminas.get_mut(actor_entity);
        if let Some(impact) = impact {
            if let Ok(mut stamina) = stamina {
                stamina.apply(impact.stamina_impact);
            }
            assert!(0 < impact.timeout.0, "{impact:?}");
            timeouts.add(actor_entity, impact.timeout);
        } else if players.get(actor_entity).is_err() {
            commands.spawn(Message::error().str("failed npc action"));
            timeouts.add(actor_entity, Milliseconds(1000));
        }
    }

    log_if_slow("manage_characters", start);
}
