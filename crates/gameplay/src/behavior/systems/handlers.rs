//! These systems are part of [`BehaviorSchedule`](`crate::behavior::schedule::BehaviorSchedule`).

use crate::behavior::systems::phrases::{Break, Heal, Hit, IsThoroughlyPulped, Kill, Pulp};
use crate::{
    Actor, ActorEvent, Amount, Clock, ContainerLimits, Corpse, CorpseEvent, CorpseRaise, Damage,
    Faction, Fragment, GameplayScreenState, Healing, Health, Item, ItemHierarchy, Life, Limited,
    LocalTerrain, LogMessageWriter, ObjectName, ObjectOn, Obstacle, Player, Shared, Stamina,
    StandardIntegrity, TerrainEvent, TileSpawner, Toggle, VisualizationUpdate, WalkingMode,
};
use bevy::ecs::schedule::{IntoScheduleConfigs as _, ScheduleConfigs};
use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::{
    Changed, ChildOf, Commands, Entity, MessageReader, NextState, ParamSet, Quat, Query, Res,
    ResMut, Transform, With, Without, on_message,
};
use cdda_json_files::{FurnitureInfo, InfoId, TerrainInfo};
use either::Either;
use gameplay_cdda::Infos;
use gameplay_location::Pos;
use std::f32::consts::FRAC_PI_2;
use std::time::Instant;
use units::Duration;
use util::log_if_slow;

pub(in super::super) fn handle_action_effects() -> ScheduleConfigs<ScheduleSystem> {
    (
        (
            // actor events
            // Make sure killed actors are handled early
            update_damaged_characters.run_if(on_message::<ActorEvent<Damage>>),
            (
                update_healed_characters.run_if(on_message::<ActorEvent<Healing>>),
                update_corpses,
            ),
        )
            .chain(),
        (
            // item events
            update_damaged_corpses.run_if(on_message::<CorpseEvent<Damage>>),
            combine_items,
        )
            .chain(),
        (
            // terrain events
            // Make sure destoyed items are handled early
            update_damaged_terrain.run_if(on_message::<TerrainEvent<Damage>>),
            toggle_doors.run_if(on_message::<TerrainEvent<Toggle>>),
        )
            .chain(),
    )
        .into_configs()
}

pub(in super::super) fn toggle_doors(
    mut commands: Commands,
    mut toggle_reader: MessageReader<TerrainEvent<Toggle>>,
    mut spawner: TileSpawner,
    mut visualization_update: ResMut<VisualizationUpdate>,
    terrain: Query<(&Pos, &Shared<TerrainInfo>, &ObjectOn)>,
) {
    let start = Instant::now();

    for toggle in toggle_reader.read() {
        let (&pos, terrain_info, &object_in) = terrain
            .get(toggle.terrain_entity)
            .expect("Terrain should be found");

        commands.entity(toggle.terrain_entity).despawn();

        let toggled = match toggle.change {
            Toggle::Open => terrain_info.open.get().expect("Terrain should be openable"),
            Toggle::Close => terrain_info
                .close
                .get()
                .expect("Terrain should be closeable"),
        };
        let local_terrain = LocalTerrain::unconnected(toggled.clone());
        spawner.spawn_terrain(object_in, pos, &local_terrain);
        *visualization_update = VisualizationUpdate::Forced;
    }

    log_if_slow("toggle_doors", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(in super::super) fn update_damaged_characters(
    mut commands: Commands,
    mut message_writer: LogMessageWriter,
    mut damage_reader: MessageReader<ActorEvent<Damage>>,
    clock: Clock,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    mut characters: Query<
        (
            &ObjectName,
            &Pos,
            &mut Health,
            &mut Transform,
            Option<&Player>,
        ),
        (With<Faction>, With<Life>),
    >,
) {
    let start = Instant::now();

    for damage in damage_reader.read() {
        let (name, pos, mut health, mut transform, player) = characters
            .get_mut(damage.actor_entity)
            .expect("Actor found");
        let evolution = health.lower(&damage.action);
        let victim = if player.is_some() {
            Fragment::you()
        } else {
            name.single(*pos)
        };
        if health.0.is_zero() {
            message_writer.send(Kill {
                killer: damage.action.attacker.clone(),
                killed: victim,
            });

            transform.rotation =
                Quat::from_rotation_y(FRAC_PI_2) * Quat::from_rotation_x(-FRAC_PI_2);
            commands
                .entity(damage.actor_entity)
                .insert((
                    Corpse,
                    CorpseRaise {
                        at: clock.time() + Duration::HOUR * 8,
                    },
                    ObjectName::corpse(),
                    StandardIntegrity(Limited::full(400)),
                ))
                .remove::<(Life, Obstacle)>();

            if player.is_some() {
                next_gameplay_state.set(GameplayScreenState::Death);
            }
        } else {
            message_writer.send(Hit {
                attacker: damage.action.attacker.clone(),
                object: victim,
                evolution,
            });
        }
    }

    log_if_slow("update_damaged_characters", start);
}

pub(in super::super) fn update_healed_characters(
    mut message_writer: LogMessageWriter,
    mut healing_reader: MessageReader<ActorEvent<Healing>>,
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
            message_writer.send(Heal {
                subject: actor.subject(),
                evolution,
            });
        }
    }

    log_if_slow("update_healed_characters", start);
}

pub(in super::super) fn update_damaged_corpses(
    mut commands: Commands,
    mut message_writer: LogMessageWriter,
    mut damage_reader: MessageReader<CorpseEvent<Damage>>,
    mut corpses: Query<(&ObjectName, &Pos, &mut StandardIntegrity), With<Corpse>>,
) {
    let start = Instant::now();

    //trace!("{} corpses found", corpses.iter().len());

    for damage in damage_reader.read() {
        let (name, pos, mut integrity) =
            corpses.get_mut(damage.corpse_entity).expect("Corpse found");
        integrity.lower(&damage.change);

        message_writer.send(Pulp {
            pulper: damage.change.attacker.clone(),
            corpse: name.single(*pos),
        });

        if integrity.0.is_zero() {
            message_writer.send(IsThoroughlyPulped {
                corpse: name.single(*pos),
            });

            commands
                .entity(damage.corpse_entity)
                .remove::<(CorpseRaise, StandardIntegrity)>();
        }
    }

    log_if_slow("update_damaged_corpses", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(in super::super) fn update_corpses(
    mut commands: Commands,
    clock: Clock,
    infos: Res<Infos>,
    mut corpse_raises: Query<(Entity, &CorpseRaise, &mut Transform)>,
) {
    let start = Instant::now();

    for (corpse, raise, mut transform) in &mut corpse_raises {
        if raise.at <= clock.time() {
            transform.rotation = Quat::IDENTITY;

            let info_id = InfoId::new("mon_zombie");
            let character_info = infos
                .characters
                .get(&info_id)
                .unwrap_or_else(|e| panic!("{info_id:?} should be found: {e:#?}"));
            let object_name = ObjectName::new(character_info.name.clone(), Faction::Zombie.color());
            let health = Health(Limited::full(character_info.hp as u16));

            commands
                .entity(corpse)
                .insert((
                    object_name,
                    Faction::Zombie,
                    Life,
                    health,
                    Stamina::Unlimited,
                    WalkingMode::Running,
                    Obstacle,
                ))
                .remove::<(Corpse, CorpseRaise, StandardIntegrity)>();
        }
    }

    log_if_slow("update_corpses", start);
}

/// For terrain and furniture
pub(in super::super) fn update_damaged_terrain(
    mut commands: Commands,
    mut message_writer: LogMessageWriter,
    mut damage_reader: MessageReader<TerrainEvent<Damage>>,
    mut spawner: TileSpawner,
    mut visualization_update: ResMut<VisualizationUpdate>,
    mut terrain: Query<(
        Entity,
        &Pos,
        &ObjectName,
        &mut StandardIntegrity,
        Option<&Shared<TerrainInfo>>,
        Option<&Shared<FurnitureInfo>>,
        &ObjectOn,
    )>,
) {
    let start = Instant::now();

    for damage in damage_reader.read() {
        let (terrain, &pos, name, mut integrity, terrain_info, furniture_info, &object_in) =
            terrain
                .get_mut(damage.terrain_entity)
                .expect("Terrain or furniture found");
        let evolution = integrity.lower(&damage.change);
        if integrity.0.is_zero() {
            message_writer.send(Break {
                breaker: damage.change.attacker.clone(),
                broken: name.single(pos),
            });
            commands.entity(terrain).despawn();
            spawner.spawn_smashed(
                object_in,
                pos,
                terrain_info
                    .map(Shared::as_ref)
                    .map(Either::Left)
                    .or_else(|| furniture_info.map(Shared::as_ref).map(Either::Right))
                    .expect("Either terrain or furniture"),
            );
            *visualization_update = VisualizationUpdate::Forced;
        } else {
            message_writer.send(Hit {
                attacker: damage.change.attacker.clone(),
                object: name.single(pos),
                evolution,
            });
        }
    }

    log_if_slow("update_damaged_items", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(in super::super) fn combine_items(
    mut commands: Commands,
    hierarchy: ItemHierarchy,
    moved_items: Query<Item, (Changed<ChildOf>, Without<ContainerLimits>)>,
) {
    let start = Instant::now();

    let mut all_merged = Vec::new();

    for moved in &moved_items {
        let has_subitems = hierarchy
            .pockets_in(&moved)
            .into_iter()
            .any(|pocket_wrapper| {
                pocket_wrapper
                    .in_pocket()
                    .is_some_and(|in_pocket| hierarchy.items_in_pocket(in_pocket).next().is_some())
            });

        if !all_merged.contains(&moved.entity) && !has_subitems {
            let mut merges = vec![];
            let mut total_amount = &Amount(0) + moved.amount;

            let siblings = match moved.parentage() {
                Either::Left(on_tile) => Either::Left(hierarchy.items_on_tile(*on_tile)),
                Either::Right(in_pocket) => Either::Right(hierarchy.items_in_pocket(*in_pocket)),
            };
            for sibling in siblings {
                // Note that the positions may differ when the parents are the same.
                if sibling.entity != moved.entity
                    && sibling.common_info.id == moved.common_info.id
                    && sibling.pos == moved.pos
                    && sibling.filthy == moved.filthy
                    && !all_merged.contains(&sibling.entity)
                {
                    merges.push(sibling.entity);
                    total_amount = &total_amount + sibling.amount;
                    all_merged.push(sibling.entity);
                }
            }

            if !merges.is_empty() {
                //trace!(
                //    "Merging {:?}/{:?} with {:?}: {} -> {}",
                //    moved.name, moved.entity, &merges, moved.amount.0, total_amount.0
                //);

                commands.entity(moved.entity).insert(total_amount);

                for merge in merges {
                    commands.entity(merge).despawn();
                }
            }
        }
    }

    log_if_slow("update_damaged_items", start);
}
