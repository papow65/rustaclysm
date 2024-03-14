use crate::prelude::*;
use bevy::prelude::*;
use std::time::Instant;

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn update_stamina(
    mut stamina_impact_events: EventReader<ActorEvent<StaminaImpact>>,
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
pub(in super::super) fn toggle_doors(
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
pub(in super::super) fn update_damaged_characters(
    mut commands: Commands,
    mut message_writer: EventWriter<Message>,
    mut damage_reader: EventReader<ActorEvent<Damage>>,
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
                .insert((
                    Corpse,
                    CorpseRaise {
                        at: clock.time() + Milliseconds::EIGHT_HOURS,
                    },
                    ObjectName::corpse(),
                    Integrity(Limited::full(400)),
                ))
                .remove::<(Life, Obstacle)>();

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
pub(in super::super) fn update_healed_characters(
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
pub(in super::super) fn update_damaged_corpses(
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
                .remove::<(CorpseRaise, Integrity)>();
        }
    }

    log_if_slow("update_damaged_corpses", start);
}

#[allow(clippy::needless_pass_by_value)]
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
            let character_info = infos.character(&definition.id).expect("Info available");
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

#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn update_explored(moved_players: Query<(), (With<Player>, Changed<Pos>)>) {
    let start = Instant::now();

    if moved_players.get_single().is_err() {
        return;
    }

    // TODO

    log_if_slow("update_corpses", start);
}

/** For terrain and furniture */
#[allow(clippy::needless_pass_by_value)]
pub(in super::super) fn update_damaged_terrain(
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
pub(in super::super) fn combine_items(
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
                let keep = *merges.iter().max().expect("'merges' should not be empty");

                /*println!(
                 *                    "Combined {} with {} others to {}",
                 *                    Phrase::from_fragments(moved_name.as_item(moved_amount, moved_filthy)),
                 *                    merges.len() - 1,
                 *                    Phrase::from_fragments(moved_name.as_item(Some(&total_amount), moved_filthy)),
                 *                );*/

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
