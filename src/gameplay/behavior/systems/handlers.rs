//! These systems are part of [`BehaviorSchedule`](`crate::gameplay::behavior::schedule::BehaviorSchedule`).

use crate::gameplay::{
    Actor, ActorEvent, Amount, Clock, ContainerLimits, Corpse, CorpseEvent, CorpseRaise, Damage,
    Faction, Fragment, GameplayScreenState, Healing, Health, Infos, Item, ItemHierarchy, Life,
    Limited, LocalTerrain, MessageWriter, ObjectName, Obstacle, Phrase, Player, Pos, Shared,
    Stamina, StandardIntegrity, Subject, TerrainEvent, Toggle, VisualizationUpdate, WalkingMode,
    spawn::TileSpawner,
};
use bevy::ecs::schedule::{IntoScheduleConfigs as _, ScheduleConfigs};
use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::{
    Changed, ChildOf, Commands, Entity, EventReader, NextState, ParamSet, Quat, Query, Res, ResMut,
    Transform, With, Without, on_event,
};
use cdda_json_files::{FurnitureInfo, InfoId, TerrainInfo};
use either::Either;
use std::time::Instant;
use units::Duration;
use util::log_if_slow;

pub(in super::super) fn handle_action_effects() -> ScheduleConfigs<ScheduleSystem> {
    (
        (
            // actor events
            // Make sure killed actors are handled early
            update_damaged_characters.run_if(on_event::<ActorEvent<Damage>>),
            (
                update_healed_characters.run_if(on_event::<ActorEvent<Healing>>),
                update_corpses,
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
            update_damaged_terrain.run_if(on_event::<TerrainEvent<Damage>>),
            toggle_doors.run_if(on_event::<TerrainEvent<Toggle>>),
        )
            .chain(),
    )
        .into_configs()
}

pub(in super::super) fn toggle_doors(
    mut commands: Commands,
    mut toggle_reader: EventReader<TerrainEvent<Toggle>>,
    mut spawner: TileSpawner,
    mut visualization_update: ResMut<VisualizationUpdate>,
    terrain: Query<(&Pos, &Shared<TerrainInfo>, &ChildOf)>,
) {
    let start = Instant::now();

    for toggle in toggle_reader.read() {
        let (&pos, terrain_info, child_of) = terrain
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
        spawner.spawn_terrain(child_of.parent(), pos, &local_terrain);
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
                    StandardIntegrity(Limited::full(400)),
                ))
                .remove::<(Life, Obstacle)>();

            if player.is_some() {
                next_gameplay_state.set(GameplayScreenState::Death);
            }
        } else {
            let mut builder = message_writer
                .subject(damage.action.attacker.clone())
                .verb("hit", "s")
                .push(victim);
            if evolution.changed() {
                builder = builder
                    .soft("for")
                    .push(Fragment::warn(format!("{}", evolution.change_abs())))
                    .soft(format!("({} -> {})", evolution.before, evolution.after));
            } else {
                builder = builder.soft("but it has").hard("no effect");
            }
            builder.send_warn();
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
            let mut builder = message_writer.subject(actor.subject()).verb("heal", "s");
            if evolution.change_abs() == 1 {
                builder = builder.push(Fragment::good("a bit"));
            } else {
                builder = builder
                    .soft("for")
                    .push(Fragment::good(format!("{}", evolution.change_abs())))
                    .soft(format!("({} -> {})", evolution.before, evolution.after));
            }
            builder.send_info();
        }
    }

    log_if_slow("update_healed_characters", start);
}

pub(in super::super) fn update_damaged_corpses(
    mut commands: Commands,
    mut message_writer: MessageWriter,
    mut damage_reader: EventReader<CorpseEvent<Damage>>,
    mut corpses: Query<(&ObjectName, &Pos, &mut StandardIntegrity), With<Corpse>>,
) {
    let start = Instant::now();

    //trace!("{} corpses found", corpses.iter().len());

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
                .hard("thoroughly pulped")
                .send_info();

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
    mut message_writer: MessageWriter,
    mut damage_reader: EventReader<TerrainEvent<Damage>>,
    mut spawner: TileSpawner,
    mut visualization_update: ResMut<VisualizationUpdate>,
    mut terrain: Query<(
        Entity,
        &Pos,
        &ObjectName,
        &mut StandardIntegrity,
        Option<&Shared<TerrainInfo>>,
        Option<&Shared<FurnitureInfo>>,
        &ChildOf,
    )>,
) {
    let start = Instant::now();

    for damage in damage_reader.read() {
        let (terrain, &pos, name, mut integrity, terrain_info, furniture_info, child_of) = terrain
            .get_mut(damage.terrain_entity)
            .expect("Terrain or furniture found");
        let evolution = integrity.lower(&damage.change);
        if integrity.0.is_zero() {
            message_writer
                .subject(damage.change.attacker.clone())
                .verb("break", "s")
                .push(name.single(pos))
                .send_warn();
            commands.entity(terrain).despawn();
            spawner.spawn_smashed(
                child_of.parent(),
                pos,
                terrain_info
                    .map(Shared::as_ref)
                    .map(Either::Left)
                    .or_else(|| furniture_info.map(Shared::as_ref).map(Either::Right))
                    .expect("Either terrain or furniture"),
            );
            *visualization_update = VisualizationUpdate::Forced;
        } else {
            let mut builder = message_writer
                .subject(damage.change.attacker.clone())
                .verb("hit", "s")
                .push(name.single(pos));
            if evolution.changed() {
                builder = builder
                    .soft("for")
                    .push(Fragment::warn(format!("{}", evolution.change_abs())))
                    .soft(format!("({} -> {})", evolution.before, evolution.after));
            } else {
                builder = builder.soft("but it has").hard("no effect");
            }
            builder.send_warn();
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
        if !all_merged.contains(&moved.entity)
            && hierarchy.items_in(moved.entity).next().is_none()
            && hierarchy.pockets_in(moved.entity).next().is_none()
        {
            let mut merges = vec![];
            let mut total_amount = &Amount(0) + moved.amount;

            for sibling in hierarchy.items_in(moved.child_of.parent()) {
                // Note that the positions may differ when the parents are the same.
                if sibling.entity != moved.entity
                    && sibling.common_info.id == moved.common_info.id
                    && sibling.pos == moved.pos
                    && sibling.filthy == moved.filthy
                    && hierarchy.items_in(sibling.entity).next().is_none()
                    && hierarchy.pockets_in(sibling.entity).next().is_none()
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
