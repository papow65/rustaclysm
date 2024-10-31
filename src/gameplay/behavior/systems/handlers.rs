//! These systems are part of [`BehaviorSchedule`](`crate::gameplay::behavior::schedule::BehaviorSchedule`).

use crate::gameplay::{
    spawn::TileSpawner, Actor, ActorEvent, Amount, Clock, ContainerLimits, Corpse, CorpseEvent,
    CorpseRaise, Damage, Faction, Fragment, GameplayScreenState, Healing, Health, Infos, Integrity,
    Item, ItemHierarchy, Life, Limited, LocalTerrain, MessageWriter, ObjectCategory,
    ObjectDefinition, ObjectName, Obstacle, Phrase, Player, Pos, Stamina, Subject, TerrainEvent,
    Toggle, VisualizationUpdate, WalkingMode,
};
use crate::util::log_if_slow;
use bevy::ecs::schedule::SystemConfigs;
use bevy::prelude::{
    on_event, Changed, Commands, DespawnRecursiveExt, Entity, EventReader, In, IntoSystem,
    IntoSystemConfigs, NextState, ParamSet, Parent, Quat, Query, Res, ResMut, Transform, With,
    Without,
};
use cdda_json_files::ObjectId;
use std::time::Instant;
use units::Duration;

pub(in super::super) fn handle_action_effects() -> SystemConfigs {
    (
        (
            // actor events
            // Make sure killed actors are handled early
            update_damaged_characters.run_if(on_event::<ActorEvent<Damage>>),
            (
                update_healed_characters.run_if(on_event::<ActorEvent<Healing>>),
                update_corpses,
                update_explored,
            ),
        )
            .chain(),
        (
            // item events
            update_damaged_corpses.run_if(on_event::<CorpseEvent<Damage>>),
            combine_items,
        )
            .chain(),
        (
            // terrain events
            // Make sure destoyed items are handled early
            update_damaged_terrain
                .pipe(spawn_broken_terrain)
                .run_if(on_event::<TerrainEvent<Damage>>),
            toggle_doors.run_if(on_event::<TerrainEvent<Toggle>>),
        )
            .chain(),
    )
        .into_configs()
}

#[expect(clippy::needless_pass_by_value)]
pub(in super::super) fn toggle_doors(
    mut commands: Commands,
    mut toggle_reader: EventReader<TerrainEvent<Toggle>>,
    infos: Res<Infos>,
    mut spawner: TileSpawner,
    mut visualization_update: ResMut<VisualizationUpdate>,
    terrain: Query<(Entity, &ObjectDefinition, &Pos, &Parent)>,
) {
    let start = Instant::now();

    for toggle in toggle_reader.read() {
        let (entity, definition, &pos, parent) = terrain.get(toggle.terrain_entity).expect("Found");

        commands.entity(entity).despawn_recursive();
        let terrain_info = infos.terrain(&definition.id);
        let toggled_id = match toggle.change {
            Toggle::Open => terrain_info.open.as_ref().expect("Openable"),
            Toggle::Close => terrain_info.close.as_ref().expect("Closeable"),
        };
        let local_terrain = LocalTerrain::unconnected(toggled_id.clone());
        spawner.spawn_terrain(&infos, parent.get(), pos, &local_terrain);
        *visualization_update = VisualizationUpdate::Forced;
    }

    log_if_slow("toggle_doors", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(in super::super) fn update_damaged_characters(
    mut commands: Commands,
    mut message_writer: MessageWriter,
    mut damage_reader: EventReader<ActorEvent<Damage>>,
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
            message_writer
                .subject(damage.action.attacker.clone())
                .verb("kill", "s")
                .push(victim)
                .send_warn();

            transform.rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)
                * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
            commands
                .entity(damage.actor_entity)
                .insert((
                    Corpse,
                    CorpseRaise {
                        at: clock.time() + Duration::HOUR * 8,
                    },
                    ObjectName::corpse(),
                    Integrity(Limited::full(400)),
                ))
                .remove::<(Life, Obstacle)>();

            if player.is_some() {
                next_gameplay_state.set(GameplayScreenState::Death);
            }
        } else {
            message_writer
                .subject(damage.action.attacker.clone())
                .verb("hit", "s")
                .push(victim)
                .add(if evolution.changed() {
                    format!(
                        "for {} ({} -> {})",
                        evolution.change_abs(),
                        evolution.before,
                        evolution.after
                    )
                } else {
                    String::from("but it has no effect")
                })
                .send_warn();
        }
    }

    log_if_slow("update_damaged_characters", start);
}

pub(in super::super) fn update_healed_characters(
    mut message_writer: MessageWriter,
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
        let evolution = health.raise(&healing.action);
        if evolution.changed() {
            let actors = actors.p1();
            let actor = actors.get(healing.actor_entity).expect("Actor found");
            message_writer
                .subject(actor.subject())
                .verb("heal", "s")
                .add(if evolution.change_abs() == 1 {
                    String::from("a bit")
                } else {
                    format!(
                        "for {} ({} -> {})",
                        evolution.change_abs(),
                        evolution.before,
                        evolution.after
                    )
                })
                .send_info();
        }
    }

    log_if_slow("update_healed_characters", start);
}

pub(in super::super) fn update_damaged_corpses(
    mut commands: Commands,
    mut message_writer: MessageWriter,
    mut damage_reader: EventReader<CorpseEvent<Damage>>,
    mut corpses: Query<(&ObjectName, &Pos, &mut Integrity), With<Corpse>>,
) {
    let start = Instant::now();

    //eprintln!("{} corpses found", corpses.iter().len());

    for damage in damage_reader.read() {
        let (name, pos, mut integrity) =
            corpses.get_mut(damage.corpse_entity).expect("Corpse found");
        integrity.lower(&damage.change);

        message_writer
            .subject(damage.change.attacker.clone())
            .verb("pulp", "s")
            .push(name.single(*pos))
            .send_info();

        if integrity.0.is_zero() {
            message_writer
                .subject(Subject::Other(Phrase::from_fragment(name.single(*pos))))
                .is()
                .add("thoroughly pulped")
                .send_info();

            commands
                .entity(damage.corpse_entity)
                .remove::<(CorpseRaise, Integrity)>();
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

            let definition = ObjectDefinition {
                category: ObjectCategory::Character,
                id: ObjectId::new("mon_zombie"),
            };
            let character_info = infos.character(&definition.id);
            let object_name = ObjectName::new(character_info.name.clone(), Faction::Zombie.color());
            let health = Health(Limited::full(character_info.hp.unwrap_or(60) as u16));

            commands
                .entity(corpse)
                .insert((
                    definition,
                    object_name,
                    Faction::Zombie,
                    Life,
                    health,
                    Stamina::Unlimited,
                    WalkingMode::Running,
                    Obstacle,
                ))
                .remove::<(Corpse, CorpseRaise, Integrity)>();
        }
    }

    log_if_slow("update_corpses", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(in super::super) fn update_explored(moved_players: Query<(), (With<Player>, Changed<Pos>)>) {
    let start = Instant::now();

    if moved_players.get_single().is_err() {
        return;
    }

    // TODO

    log_if_slow("update_corpses", start);
}

/// For terrain and furniture
pub(in super::super) fn update_damaged_terrain(
    mut commands: Commands,
    mut message_writer: MessageWriter,
    mut damage_reader: EventReader<TerrainEvent<Damage>>,
    mut visualization_update: ResMut<VisualizationUpdate>,
    mut terrain: Query<(
        Entity,
        &Pos,
        &ObjectName,
        &mut Integrity,
        &ObjectDefinition,
        &Parent,
    )>,
) -> Vec<(Entity, Pos, ObjectDefinition)> {
    let start = Instant::now();

    let mut broken = Vec::new();

    for damage in damage_reader.read() {
        let (terrain, &pos, name, mut integrity, definition, parent) = terrain
            .get_mut(damage.terrain_entity)
            .expect("Terrain or furniture found");
        let evolution = integrity.lower(&damage.change);
        if integrity.0.is_zero() {
            message_writer
                .subject(damage.change.attacker.clone())
                .verb("break", "s")
                .push(name.single(pos))
                .send_warn();
            commands.entity(terrain).despawn_recursive();
            broken.push((parent.get(), pos, definition.clone()));
            *visualization_update = VisualizationUpdate::Forced;
        } else {
            message_writer
                .subject(damage.change.attacker.clone())
                .verb("hit", "s")
                .push(name.single(pos))
                .add(if evolution.changed() {
                    format!(
                        "for {} ({} -> {})",
                        evolution.change_abs(),
                        evolution.before,
                        evolution.after
                    )
                } else {
                    String::from("but it has no effect")
                })
                .send_warn();
        }
    }

    log_if_slow("update_damaged_items", start);

    broken
}

// Separate from 'update_damaged_terrain' to prevent a conflict with 'Location'.
/// For terrain and furniture
#[expect(clippy::needless_pass_by_value)]
pub(in super::super) fn spawn_broken_terrain(
    In(broken): In<Vec<(Entity, Pos, ObjectDefinition)>>,
    mut spawner: TileSpawner,
    infos: Res<Infos>,
) {
    let start = Instant::now();

    for (parent_entity, pos, definition) in broken {
        spawner.spawn_smashed(&infos, parent_entity, pos, &definition);
    }

    log_if_slow("spawn_broken_terrain", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(in super::super) fn combine_items(
    mut commands: Commands,
    hierarchy: ItemHierarchy,
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
                let keep = *merges.iter().max().expect("'merges' should not be empty");

                //println!(
                //    "Combined {} with {} others to {}",
                //    Phrase::from_fragments(moved_name.as_item(moved_amount, moved_filthy)),
                //    merges.len() - 1,
                //    Phrase::from_fragments(moved_name.as_item(Some(&total_amount), moved_filthy)),
                //);

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
