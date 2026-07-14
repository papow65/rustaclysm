use crate::actor::messages::{
    FirstExamineYourDestination, NoPlaceToCraftNearby, NoTargetsNearby, NothingToCloseNearby,
    YouAreAlmostOutOfBreathAndStop, YouAreStillAsleep, YouAreStillDraggingItems, YouCant,
    YouCantAttackYourself, YouFallAsleep, YouFinish, YouSpotAndStop, YouStartDefending,
    YouWakeUpAfterSleeping,
};
use crate::{
    ActorItem, Breath, ContinueCraft, Faction, Intelligence, Interruption, MoveItem, Pathfinder,
    PlannedAction, PlayerDirection, PlayerInstructions, Pulp, QueuedInstruction, StartCraft,
};
use bevy::prelude::{DetectChanges as _, Entity, NextState, ResMut};
use gameplay_crafting::RecipeSituation;
use gameplay_location::{CardinalDirection, HorizontalDirection, Nbor, Pos, VisionDistance};
use gameplay_log::{LogMessageWriter, Severity};
use gameplay_player::{PickingNbor, PlayerActionState};
use gameplay_world::{CurrentlyVisibleBuilder, Envir, Explored};
use text::Fragment;
use units::{Duration, Timestamp};

pub(crate) fn plan_manual_action(
    current_state: &PlayerActionState,
    next_state: &mut ResMut<NextState<PlayerActionState>>,
    message_writer: &mut LogMessageWriter,
    envir: &Envir,
    player_instructions: &mut PlayerInstructions,
    player: &ActorItem,
    now: Timestamp,
) -> Option<PlannedAction> {
    while let Some(instruction) = player_instructions.pop() {
        if let Some(action) = plan(
            current_state,
            next_state,
            message_writer,
            envir,
            *player.pos,
            instruction,
            now,
        ) {
            return Some(action);
        } else {
            // The action was either invalid or one that took no time (like [`PlannedAction::ExamineItem`].
            // Continue with next instruction
        }
    }

    None
}

pub(crate) fn plan_automatic_action(
    current_state: &PlayerActionState,
    currently_visible_builder: &CurrentlyVisibleBuilder,
    player_instructions: &mut PlayerInstructions,
    explored: &Explored,
    player: &ActorItem,
    now: Timestamp,
    factions: &[(Pos, &Faction)],
) -> Option<PlannedAction> {
    let envir = &currently_visible_builder.envir;
    let enemy_name = player
        .faction
        .enemy_name(currently_visible_builder, factions, player);
    match current_state {
        PlayerActionState::Dragging { from } => {
            plan_auto_drag(envir, player_instructions, from, enemy_name)
        }
        PlayerActionState::Crafting { item } => {
            plan_auto_continue_craft(player_instructions, *item, enemy_name)
        }
        PlayerActionState::AutoDefend => {
            let enemies = player
                .faction
                .enemies(currently_visible_builder, factions, player);
            plan_auto_defend(envir, player_instructions, player, &enemies)
        }
        PlayerActionState::AutoTravel { target } => plan_auto_travel(
            envir,
            player_instructions,
            explored,
            player,
            target,
            enemy_name,
        ),
        PlayerActionState::Pulping { direction } => {
            plan_auto_pulp(envir, player_instructions, player, *direction, enemy_name)
        }
        PlayerActionState::Waiting { until } => {
            plan_auto_wait(player_instructions, now, until, enemy_name)
        }
        PlayerActionState::Sleeping { from } => plan_auto_sleep(player_instructions, now, from),
        _ => None,
    }
}

fn plan(
    current_state: &PlayerActionState,
    next_state: &mut ResMut<NextState<PlayerActionState>>,
    message_writer: &mut LogMessageWriter,
    envir: &Envir,
    player_pos: Pos,
    instruction: QueuedInstruction,
    now: Timestamp,
) -> Option<PlannedAction> {
    match (current_state, instruction) {
        (
            PlayerActionState::Sleeping { from },
            QueuedInstruction::Interrupt(Interruption::Finished),
        ) => {
            message_writer.send(YouWakeUpAfterSleeping {
                duration: now - *from,
            });
            next_state.set(PlayerActionState::Normal);
            None
        }
        (PlayerActionState::Sleeping { .. }, _) => {
            // Can not be interrupted
            message_writer.send(YouAreStillAsleep);
            None
        }
        (PlayerActionState::Peeking { .. }, instruction) => stop_peeking(
            next_state,
            message_writer,
            envir,
            player_pos,
            instruction,
            now,
        ),
        (
            PlayerActionState::Normal | PlayerActionState::PickingNbor(PickingNbor::Dragging),
            QueuedInstruction::Offset(PlayerDirection::Here),
        ) => Some(PlannedAction::Stay),
        (PlayerActionState::Normal, QueuedInstruction::Sleep) => {
            message_writer.send(YouFallAsleep);
            next_state.set(PlayerActionState::Sleeping { from: now });
            None
        }
        (PlayerActionState::Normal, QueuedInstruction::ToggleAutoDefend) => {
            next_state.set(PlayerActionState::AutoDefend);
            message_writer.send(YouStartDefending);
            None
        }
        (_, QueuedInstruction::ToggleAutoTravel) => {
            message_writer.send(FirstExamineYourDestination);
            None
        }
        (
            PlayerActionState::PickingNbor(PickingNbor::Attacking),
            QueuedInstruction::Offset(PlayerDirection::Here),
        ) => {
            message_writer.send(YouCantAttackYourself);
            None
        }
        (PlayerActionState::PickingNbor(PickingNbor::Attacking), QueuedInstruction::Attack)
        | (PlayerActionState::PickingNbor(PickingNbor::Smashing), QueuedInstruction::Smash)
        | (PlayerActionState::PickingNbor(PickingNbor::Peeking), QueuedInstruction::Peek)
        | (
            PlayerActionState::PickingNbor(PickingNbor::Dragging)
            | PlayerActionState::Dragging { .. },
            QueuedInstruction::Drag | QueuedInstruction::CancelAction,
        )
        | (
            PlayerActionState::PickingNbor(PickingNbor::Pulping)
            | PlayerActionState::Pulping { .. },
            QueuedInstruction::Interrupt(Interruption::Finished),
        ) => {
            next_state.set(PlayerActionState::Normal);
            None
        }
        (PlayerActionState::Dragging { .. }, instruction) => {
            match instruction {
                QueuedInstruction::Interrupt(Interruption::Finished) => {
                    next_state.set(PlayerActionState::PickingNbor(PickingNbor::Dragging));
                }
                QueuedInstruction::Interrupt(Interruption::Danger(danger)) => {
                    message_writer.send(YouSpotAndStop {
                        seen: danger,
                        verb: "dragging",
                    });
                    next_state.set(PlayerActionState::Normal);
                }
                _ => message_writer.send(YouAreStillDraggingItems),
            }
            None
        }
        (_, instruction) => generic_plan(
            current_state,
            next_state,
            message_writer,
            envir,
            player_pos,
            instruction,
        ),
    }
}

/// For plans that not primarily depend on [`PlayerActionState`]
fn generic_plan(
    current_state: &PlayerActionState,
    next_state: &mut ResMut<NextState<PlayerActionState>>,
    message_writer: &mut LogMessageWriter,
    envir: &Envir,
    player_pos: Pos,
    instruction: QueuedInstruction,
) -> Option<PlannedAction> {
    //trace!("processing generic instruction: {instruction:?}");
    match instruction {
        QueuedInstruction::CancelAction
        | QueuedInstruction::Sleep
        | QueuedInstruction::ToggleAutoTravel
        | QueuedInstruction::ToggleAutoDefend => {
            next_state.set(PlayerActionState::Normal);
            None
        }
        QueuedInstruction::Offset(direction) => handle_offset(
            current_state,
            next_state,
            message_writer,
            envir,
            player_pos,
            direction.to_nbor(),
        ),
        QueuedInstruction::Wield(wield) => Some(PlannedAction::Wield(wield)),
        QueuedInstruction::Unwield(unwield) => Some(PlannedAction::Unwield(unwield)),
        QueuedInstruction::Pickup(pickup) => Some(PlannedAction::Pickup(pickup)),
        QueuedInstruction::MoveItem(move_item) => Some(PlannedAction::MoveItem(move_item)),
        QueuedInstruction::StartCraft(recipe_situation) => plan_start_craft(
            next_state,
            message_writer,
            envir,
            player_pos,
            recipe_situation,
        ),
        // TODO instruction to continue crafting
        QueuedInstruction::Attack => plan_attack(next_state, message_writer, envir, player_pos),
        QueuedInstruction::Smash => plan_smash(next_state, message_writer, envir, player_pos),
        QueuedInstruction::Pulp => plan_pulp(next_state, message_writer, envir, player_pos),
        QueuedInstruction::Peek => {
            next_state.set(PlayerActionState::PickingNbor(PickingNbor::Peeking));
            None
        }
        QueuedInstruction::Close => plan_close(next_state, message_writer, envir, player_pos),
        QueuedInstruction::Drag => {
            next_state.set(PlayerActionState::PickingNbor(PickingNbor::Dragging));
            None
        }
        QueuedInstruction::ExamineItem(examine_item) => {
            Some(PlannedAction::ExamineItem(examine_item))
        }
        QueuedInstruction::ChangePace(change_pace) => Some(PlannedAction::ChangePace(change_pace)),
        QueuedInstruction::Interrupt(Interruption::Danger(fragment)) => {
            message_writer.send(YouSpotAndStop {
                seen: fragment,
                verb: current_state.to_string().to_lowercase(),
            });
            next_state.set(PlayerActionState::Normal);
            None
        }
        QueuedInstruction::Interrupt(Interruption::LowStamina) => {
            message_writer.send(YouAreAlmostOutOfBreathAndStop {
                verb: current_state.to_string().to_lowercase(),
            });
            next_state.set(PlayerActionState::Normal);
            None
        }
        QueuedInstruction::Interrupt(Interruption::Finished) => {
            next_state.set(PlayerActionState::Normal);
            match current_state.severity_finishing() {
                Severity::Neutral => message_writer.send(YouFinish::<false> {
                    action: current_state.clone(),
                }),
                Severity::Success => message_writer.send(YouFinish::<true> {
                    action: current_state.clone(),
                }),
                other => unimplemented!("{other:?}"),
            }
            None
        }
    }
}

fn handle_offset(
    current_state: &PlayerActionState,
    next_state: &mut ResMut<NextState<PlayerActionState>>,
    message_writer: &mut LogMessageWriter,
    envir: &Envir,
    player_pos: Pos,
    raw_nbor: Nbor,
) -> Option<PlannedAction> {
    if !matches!(*current_state, PlayerActionState::Sleeping { .. })
        && let Err(failure) = envir.get_nbor(player_pos, raw_nbor)
    {
        message_writer.send(failure);
        return None;
    }

    match &current_state {
        PlayerActionState::Sleeping { .. }
        | PlayerActionState::Peeking { .. }
        | PlayerActionState::Dragging { .. }
        | PlayerActionState::Pulping { .. } => {
            panic!("{current_state:?} {player_pos:?} {raw_nbor:?}");
        }
        PlayerActionState::Crafting { .. }
        | PlayerActionState::Waiting { .. }
        | PlayerActionState::AutoTravel { .. }
        | PlayerActionState::AutoDefend => {
            next_state.set(PlayerActionState::Normal);
            None
        }
        PlayerActionState::Normal => Some(PlannedAction::step(raw_nbor)),
        PlayerActionState::PickingNbor(picking_nbor) => {
            match picking_nbor {
                PickingNbor::Attacking => {
                    next_state.set(PlayerActionState::Normal);
                    Some(PlannedAction::attack(raw_nbor))
                }
                PickingNbor::Smashing => {
                    next_state.set(PlayerActionState::Normal);
                    Some(PlannedAction::smash(raw_nbor))
                }
                PickingNbor::Pulping => {
                    //trace!("Inactive pulping");
                    if let Nbor::Horizontal(target) = raw_nbor {
                        //trace!("Activating pulping");
                        Some(PlannedAction::pulp(target))
                    } else {
                        message_writer.send(YouCant {
                            verb: "pulp",
                            direction: "vertically",
                        });
                        None
                    }
                }
                PickingNbor::Peeking => handle_peeking_offset(next_state, message_writer, raw_nbor),
                PickingNbor::Closing => {
                    next_state.set(PlayerActionState::Normal);
                    if let Nbor::Horizontal(target) = raw_nbor {
                        Some(PlannedAction::close(target))
                    } else {
                        message_writer.send(YouCant {
                            verb: "close",
                            direction: "vertically",
                        });
                        None
                    }
                }
                PickingNbor::Dragging => {
                    next_state.set(PlayerActionState::Dragging { from: player_pos });
                    Some(PlannedAction::step(raw_nbor))
                }
                PickingNbor::Crafting(recipe_situation) => {
                    if let Nbor::Horizontal(target) = raw_nbor {
                        // next_state is set when performing the action
                        Some(PlannedAction::StartCraft(StartCraft {
                            recipe_situation: recipe_situation.clone(),
                            target,
                        }))
                    } else {
                        message_writer.send(YouCant {
                            verb: "craft",
                            direction: "vertically",
                        });
                        None
                    }
                }
            }
        }
    }
}

fn plan_start_craft(
    next_state: &mut ResMut<NextState<PlayerActionState>>,
    message_writer: &mut LogMessageWriter,
    envir: &Envir,
    pos: Pos,
    recipe_situation: RecipeSituation,
) -> Option<PlannedAction> {
    let start_craft = QueuedInstruction::StartCraft(recipe_situation);
    let craftable_nbors = Pathfinder::new(envir)
        .nbors_for_exploring(pos, &start_craft)
        .collect::<Vec<_>>();
    let QueuedInstruction::StartCraft(recipe_situation) = start_craft else {
        panic!("The instruction {start_craft:?} should still match start craft");
    };

    match craftable_nbors.len() {
        0 => {
            message_writer.send(NoPlaceToCraftNearby);
            None
        }
        1 => {
            if let Nbor::Horizontal(horizontal_direction) =
                craftable_nbors.first().expect("Single valid pos")
            {
                // Craftig state is set when performing the action
                Some(PlannedAction::StartCraft(StartCraft {
                    recipe_situation,
                    target: *horizontal_direction,
                }))
            } else {
                message_writer.send(NoPlaceToCraftNearby);
                None
            }
        }
        _ => {
            next_state.set(PlayerActionState::PickingNbor(PickingNbor::Crafting(
                recipe_situation,
            )));
            None
        }
    }
}

fn plan_attack(
    next_state: &mut ResMut<NextState<PlayerActionState>>,
    message_writer: &mut LogMessageWriter,
    envir: &Envir,
    pos: Pos,
) -> Option<PlannedAction> {
    let attackable_nbors = Pathfinder::new(envir)
        .nbors_for_exploring(pos, &QueuedInstruction::Attack)
        .collect::<Vec<_>>();
    match attackable_nbors.len() {
        0 => {
            message_writer.send(NoTargetsNearby);
            None
        }
        1 => Some(PlannedAction::attack(attackable_nbors[0])),
        _ => {
            next_state.set(PlayerActionState::PickingNbor(PickingNbor::Attacking));
            None
        }
    }
}

fn plan_smash(
    next_state: &mut ResMut<NextState<PlayerActionState>>,
    message_writer: &mut LogMessageWriter,
    envir: &Envir,
    pos: Pos,
) -> Option<PlannedAction> {
    let smashable_nbors = Pathfinder::new(envir)
        .nbors_for_exploring(pos, &QueuedInstruction::Smash)
        .collect::<Vec<_>>();
    match smashable_nbors.len() {
        0 => {
            message_writer.send(NoTargetsNearby);
            None
        }
        1 => Some(PlannedAction::smash(smashable_nbors[0])),
        _ => {
            next_state.set(PlayerActionState::PickingNbor(PickingNbor::Smashing));
            None
        }
    }
}

fn plan_pulp(
    next_state: &mut ResMut<NextState<PlayerActionState>>,
    message_writer: &mut LogMessageWriter,
    envir: &Envir,
    pos: Pos,
) -> Option<PlannedAction> {
    let pulpable_nbors = Pathfinder::new(envir)
        .nbors_for_exploring(pos, &QueuedInstruction::Pulp)
        .filter_map(|nbor| {
            if let Nbor::Horizontal(horizontal) = nbor {
                Some(horizontal)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    //trace!("Pulping {} targets", pulpable_nbors.len());
    match pulpable_nbors.len() {
        0 => {
            message_writer.send(NoTargetsNearby);
            None
        }
        1 => {
            //trace!("Pulping target found -> active");
            next_state.set(PlayerActionState::Pulping {
                direction: pulpable_nbors[0],
            });
            Some(PlannedAction::pulp(pulpable_nbors[0]))
        }
        _ => {
            //trace!("Pulping choice -> inactive");
            next_state.set(PlayerActionState::PickingNbor(PickingNbor::Pulping));
            None
        }
    }
}

fn plan_close(
    next_state: &mut ResMut<NextState<PlayerActionState>>,
    message_writer: &mut LogMessageWriter,
    envir: &Envir,
    pos: Pos,
) -> Option<PlannedAction> {
    let closable_nbors = Pathfinder::new(envir)
        .nbors_for_exploring(pos, &QueuedInstruction::Close)
        .filter_map(|nbor| {
            if let Nbor::Horizontal(horizontal) = nbor {
                Some(horizontal)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    match closable_nbors.len() {
        0 => {
            message_writer.send(NothingToCloseNearby);
            None
        }
        1 => Some(PlannedAction::close(closable_nbors[0])),
        _ => {
            next_state.set(PlayerActionState::PickingNbor(PickingNbor::Closing));
            None
        }
    }
}

fn stop_peeking(
    next_state: &mut ResMut<NextState<PlayerActionState>>,
    message_writer: &mut LogMessageWriter,
    envir: &Envir,
    player_pos: Pos,
    instruction: QueuedInstruction,
    now: Timestamp,
) -> Option<PlannedAction> {
    // Pretend to be Self::Normal - call the standalone plan function
    plan(
        &PlayerActionState::Normal,
        next_state,
        message_writer,
        envir,
        player_pos,
        instruction,
        now,
    )
    .inspect(
        // Only called for Some(planned_action)
        |_| {
            if !next_state.is_changed() {
                next_state.set(PlayerActionState::Normal);
            }
        },
    )
}

fn handle_peeking_offset(
    next_state: &mut ResMut<NextState<PlayerActionState>>,
    message_writer: &mut LogMessageWriter,
    raw_nbor: Nbor,
) -> Option<PlannedAction> {
    match raw_nbor {
        Nbor::Up | Nbor::Down => {
            message_writer.send(YouCant {
                verb: "peek",
                direction: "vertically",
            });
            None
        }
        Nbor::HERE => {
            message_writer.send(YouCant {
                verb: "peek",
                direction: "here",
            });
            None
        }
        Nbor::Horizontal(
            HorizontalDirection::NorthWest
            | HorizontalDirection::SouthWest
            | HorizontalDirection::NorthEast
            | HorizontalDirection::SouthEast,
        ) => {
            message_writer.send(YouCant {
                verb: "peek",
                direction: "diagonally",
            });
            None
        }
        Nbor::Horizontal(
            horizontal_direction @ (HorizontalDirection::North
            | HorizontalDirection::South
            | HorizontalDirection::East
            | HorizontalDirection::West),
        ) => {
            let target = CardinalDirection::try_from(horizontal_direction).unwrap_or_else(|()| {
                panic!("{horizontal_direction:?} should match a cardinal direction");
            });
            //trace!("Activating peeking");
            // Not PlayerActionState::Peeking, because that is not validated yet.
            next_state.set(PlayerActionState::Normal);
            Some(PlannedAction::peek(target))
        }
    }
}

fn plan_auto_drag(
    envir: &Envir<'_, '_>,
    player_instructions: &mut PlayerInstructions,
    from: &Pos,
    enemy_name: Option<Fragment>,
) -> Option<PlannedAction> {
    if let Some(item) = envir.find_item(*from) {
        interrupt_on_danger(
            player_instructions,
            enemy_name,
            PlannedAction::MoveItem(MoveItem {
                item_entity: item.entity,
                to: Nbor::HERE,
            }),
        )
    } else {
        player_instructions.interrupt(Interruption::Finished);
        None
    }
}

fn plan_auto_continue_craft(
    player_instructions: &mut PlayerInstructions,
    item_entity: Entity,
    enemy_name: Option<Fragment>,
) -> Option<PlannedAction> {
    // Finishing a craft is checked when performing the action
    interrupt_on_danger(
        player_instructions,
        enemy_name,
        PlannedAction::ContinueCraft(ContinueCraft { item_entity }),
    )
}

fn plan_auto_travel(
    envir: &Envir<'_, '_>,
    player_instructions: &mut PlayerInstructions,
    explored: &Explored,
    player: &ActorItem<'_, '_>,
    target: &Pos,
    enemy_name: Option<Fragment>,
) -> Option<PlannedAction> {
    if *player.pos == *target {
        player_instructions.interrupt(Interruption::Finished);
        None
    } else if let Some(enemy_name) = enemy_name {
        player_instructions.interrupt(Interruption::Danger(enemy_name));
        None
    } else if player.stamina.breath() != Breath::Normal {
        player_instructions.interrupt(Interruption::LowStamina);
        None
    } else {
        Pathfinder::new(envir)
            .path(
                *player.pos,
                *target,
                Intelligence::Smart,
                |pos| explored.has_pos_been_seen(pos),
                player.speed(),
                player.stay_duration(),
            )
            .map(|path| {
                envir
                    .nbor(*player.pos, path.first)
                    .expect("The first step should be a nbor")
            })
            .or_else(|| {
                // Full path not available
                Pathfinder::new(envir)
                    .nbors_for_moving(*player.pos, None, Intelligence::Smart, player.speed())
                    .map(|(nbor, nbor_pos, _)| (nbor, nbor_pos.vision_distance(*target)))
                    .min_by_key(|(_, distance)| distance.as_tiles())
                    .filter(|(_, distance)| {
                        distance.in_range(VisionDistance::MAX_VISION_TILES as usize)
                    })
                    .map(|(nbor, _)| nbor)
            })
            .map(PlannedAction::step)
    }
}

fn plan_auto_defend(
    envir: &Envir<'_, '_>,
    player_instructions: &mut PlayerInstructions,
    player: &ActorItem<'_, '_>,
    enemies: &[Pos],
) -> Option<PlannedAction> {
    if enemies.is_empty() {
        player_instructions.interrupt(Interruption::Finished);
        None
    } else if player.stamina.breath() != Breath::Normal {
        player_instructions.interrupt(Interruption::LowStamina);
        None
    } else if let Some(target) = enemies.iter().find_map(|pos| envir.nbor(*player.pos, *pos)) {
        Some(PlannedAction::attack(target))
    } else {
        Some(PlannedAction::Stay)
    }
}

fn plan_auto_pulp(
    envir: &Envir<'_, '_>,
    player_instructions: &mut PlayerInstructions,
    player: &ActorItem<'_, '_>,
    target: HorizontalDirection,
    enemy_name: Option<Fragment>,
) -> Option<PlannedAction> {
    //trace!("Post instruction pulp handling...");
    if envir
        .find_pulpable(player.pos.horizontal_nbor(target))
        .is_some()
    {
        interrupt_on_danger(
            player_instructions,
            enemy_name,
            if player.stamina.breath() == Breath::Normal {
                //trace!("Keep pulping");
                PlannedAction::Pulp(Pulp { target })
            } else {
                //trace!("Keep pulping after catching breath");
                PlannedAction::Stay
            },
        )
    } else {
        //trace!("Stop pulping");
        player_instructions.interrupt(Interruption::Finished);
        None
    }
}

fn plan_auto_wait(
    player_instructions: &mut PlayerInstructions,
    now: Timestamp,
    until: &Timestamp,
    enemy_name: Option<Fragment>,
) -> Option<PlannedAction> {
    if *until <= now {
        player_instructions.interrupt(Interruption::Finished);
        None
    } else {
        interrupt_on_danger(player_instructions, enemy_name, PlannedAction::Stay)
    }
}

fn interrupt_on_danger(
    player_instructions: &mut PlayerInstructions,
    enemy_name: Option<Fragment>,
    planned_action: PlannedAction,
) -> Option<PlannedAction> {
    if let Some(enemy_name) = enemy_name {
        player_instructions.interrupt(Interruption::Danger(enemy_name));
        None
    } else {
        Some(planned_action)
    }
}

fn plan_auto_sleep(
    player_instructions: &mut PlayerInstructions,
    now: Timestamp,
    from: &Timestamp,
) -> Option<PlannedAction> {
    // TODO interrupt on taking damage

    if *from + Duration::HOUR * 8 <= now {
        player_instructions.interrupt(Interruption::Finished);
        None
    } else {
        Some(PlannedAction::Sleep)
    }
}
