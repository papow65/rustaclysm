use crate::prelude::*;
use bevy::prelude::{Color, Component, DetectChanges, Entity, NextState, ResMut, States};
use std::fmt;

#[derive(Debug, Component)]
pub(crate) struct Player;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum PickingNbor {
    Attacking,
    Smashing,
    Pulping,
    Peeking,
    Closing,
    Dragging,
    Crafting { recipe_id: ObjectId },
}

/// Current action of the player character
/// Conceptually, this is a child state of [`GameplayScreenState::Base`].
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, States)]
pub(crate) enum PlayerActionState {
    #[default]
    Normal,
    PickingNbor(PickingNbor),
    Pulping {
        direction: HorizontalDirection,
    },
    Peeking {
        direction: CardinalDirection,
    },
    Dragging {
        from: Pos,
    },
    Crafting {
        /// The craft item, not the resulting item
        item: Entity,
    },
    Waiting {
        until: Timestamp,
    },
    Sleeping {
        until: Timestamp,
    },
    AutoTravel {
        target: Pos,
    },
    AutoDefend,
}

impl PlayerActionState {
    fn start_waiting(now: Timestamp) -> Self {
        Self::Waiting {
            until: now + Duration::MINUTE,
        }
    }

    fn start_sleeping(now: Timestamp) -> Self {
        Self::Sleeping {
            until: now + Duration::HOUR * 8,
        }
    }

    pub(crate) fn plan_manual_action(
        &self,
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        instruction_queue: &mut InstructionQueue,
        player: &ActorItem,
        now: Timestamp,
    ) -> Option<PlannedAction> {
        while let Some(instruction) = instruction_queue.pop() {
            if let Some(action) = self.plan(
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
        &self,
        currently_visible_builder: &CurrentlyVisibleBuilder,
        instruction_queue: &mut InstructionQueue,
        explored: &Explored,
        player: &ActorItem,
        now: Timestamp,
        factions: &[(Pos, &Faction)],
    ) -> Option<PlannedAction> {
        let envir = &currently_visible_builder.envir;
        let enemy_name = player
            .faction
            .enemy_name(currently_visible_builder, factions, player);
        match self {
            Self::Dragging { from } => plan_auto_drag(envir, instruction_queue, from, enemy_name),
            Self::Crafting { item } => {
                plan_auto_continue_craft(instruction_queue, *item, enemy_name)
            }
            Self::AutoDefend => {
                let enemies = player
                    .faction
                    .enemies(currently_visible_builder, factions, player);
                plan_auto_defend(envir, instruction_queue, player, &enemies)
            }
            Self::AutoTravel { target } => plan_auto_travel(
                envir,
                instruction_queue,
                explored,
                player,
                *target,
                enemy_name,
            ),
            Self::Pulping { direction } => {
                plan_auto_pulp(envir, instruction_queue, player, *direction, enemy_name)
            }
            Self::Waiting { until } => plan_auto_wait(instruction_queue, now, until, enemy_name),
            Self::Sleeping { until } => plan_auto_sleep(instruction_queue, now, until),
            _ => None,
        }
    }

    fn plan(
        &self,
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        player_pos: Pos,
        instruction: QueuedInstruction,
        now: Timestamp,
    ) -> Option<PlannedAction> {
        println!("processing instruction: {instruction:?}");
        match (&self, instruction) {
            (Self::Sleeping { .. }, QueuedInstruction::Interrupt(Interruption::Finished)) => {
                next_state.set(Self::Normal);
                message_writer.you("wake up").send_info();
                None
            }
            (Self::Sleeping { .. }, _) => {
                // Can not be interrupted
                message_writer.you("are still asleep. Zzz...").send_error();
                None
            }
            (Self::Peeking { .. }, instruction) => Self::stop_peeking(
                next_state,
                message_writer,
                envir,
                player_pos,
                instruction,
                now,
            ),
            (
                Self::Normal | Self::PickingNbor(PickingNbor::Dragging),
                QueuedInstruction::Offset(PlayerDirection::Here),
            ) => Some(PlannedAction::Stay),
            (Self::Normal, QueuedInstruction::Wait) => {
                next_state.set(Self::start_waiting(now));
                message_writer.you("wait...").send_info();
                None
            }
            (Self::Normal, QueuedInstruction::Sleep) => {
                next_state.set(Self::start_sleeping(now));
                message_writer.you("fall asleep... Zzz...").send_info();
                None
            }
            (Self::Normal, QueuedInstruction::ToggleAutoDefend) => {
                next_state.set(Self::AutoDefend);
                message_writer.you("start defending...").send_warn();
                None
            }
            (_, QueuedInstruction::ToggleAutoTravel) => {
                message_writer
                    .str("First examine your destination")
                    .send_error();
                None
            }
            (
                Self::PickingNbor(PickingNbor::Attacking),
                QueuedInstruction::Offset(PlayerDirection::Here),
            ) => {
                message_writer.you("can't attack yourself").send_error();
                None
            }
            (Self::PickingNbor(PickingNbor::Attacking), QueuedInstruction::Attack)
            | (Self::PickingNbor(PickingNbor::Smashing), QueuedInstruction::Smash)
            | (Self::PickingNbor(PickingNbor::Peeking), QueuedInstruction::Peek)
            | (
                Self::PickingNbor(PickingNbor::Dragging) | Self::Dragging { .. },
                QueuedInstruction::Drag | QueuedInstruction::CancelAction,
            )
            | (
                Self::PickingNbor(PickingNbor::Pulping) | Self::Pulping { .. },
                QueuedInstruction::Interrupt(Interruption::Finished),
            ) => {
                next_state.set(Self::Normal);
                None
            }
            (Self::Dragging { .. }, instruction) => {
                match instruction {
                    QueuedInstruction::Interrupt(Interruption::Finished) => {
                        next_state.set(Self::PickingNbor(PickingNbor::Dragging));
                    }
                    QueuedInstruction::Interrupt(_) => {
                        message_writer
                            .you("spot an enemy and stop dragging")
                            .send_warn();
                        next_state.set(Self::Normal);
                    }
                    _ => message_writer.you("are still dragging items").send_warn(),
                }
                None
            }
            (_, instruction) => {
                self.generic_plan(next_state, message_writer, envir, player_pos, instruction)
            }
        }
    }

    /// For plans that not depend on self
    #[allow(clippy::needless_pass_by_value)]
    fn generic_plan(
        &self,
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        player_pos: Pos,
        instruction: QueuedInstruction,
    ) -> Option<PlannedAction> {
        //println!("processing generic instruction: {instruction:?}");
        match instruction {
            QueuedInstruction::CancelAction
            | QueuedInstruction::Wait
            | QueuedInstruction::Sleep
            | QueuedInstruction::ToggleAutoTravel
            | QueuedInstruction::ToggleAutoDefend => {
                next_state.set(Self::Normal);
                None
            }
            QueuedInstruction::Offset(direction) => self.handle_offset(
                next_state,
                message_writer,
                envir,
                player_pos,
                direction.to_nbor(),
            ),
            QueuedInstruction::Wield(item) => Some(PlannedAction::Wield { item }),
            QueuedInstruction::Unwield(item) => Some(PlannedAction::Unwield { item }),
            QueuedInstruction::Pickup(item) => Some(PlannedAction::Pickup { item }),
            QueuedInstruction::Dump(item, direction) => Some(PlannedAction::MoveItem {
                item,
                to: Nbor::Horizontal(direction),
            }),
            QueuedInstruction::StartCraft { recipe_id } => {
                Self::handle_start_craft(next_state, message_writer, envir, player_pos, recipe_id)
            }
            // TODO instruction to continue crafting
            QueuedInstruction::Attack => {
                Self::handle_attack(next_state, message_writer, envir, player_pos)
            }
            QueuedInstruction::Smash => {
                Self::handle_smash(next_state, message_writer, envir, player_pos)
            }
            QueuedInstruction::Pulp => {
                Self::handle_pulp(next_state, message_writer, envir, player_pos)
            }
            QueuedInstruction::Peek => {
                next_state.set(Self::PickingNbor(PickingNbor::Peeking));
                None
            }
            QueuedInstruction::Close => {
                Self::handle_close(next_state, message_writer, envir, player_pos)
            }
            QueuedInstruction::Drag => {
                next_state.set(Self::PickingNbor(PickingNbor::Dragging));
                None
            }
            QueuedInstruction::ExamineItem(item) => Some(PlannedAction::ExamineItem { item }),
            QueuedInstruction::ChangePace => Some(PlannedAction::ChangePace),
            QueuedInstruction::Interrupt(Interruption::Danger(fragments)) => {
                next_state.set(Self::Normal);
                message_writer
                    .you("spot")
                    .push(fragments)
                    .add("and stop")
                    .add(self.to_string().to_lowercase())
                    .send_warn();
                None
            }
            QueuedInstruction::Interrupt(Interruption::LowStamina) => {
                next_state.set(Self::Normal);
                message_writer
                    .you("are almost out of breath and stop")
                    .add(self.to_string().to_lowercase())
                    .send_warn();
                None
            }
            QueuedInstruction::Interrupt(Interruption::Finished) => {
                next_state.set(Self::Normal);
                message_writer
                    .you("finish")
                    .add(if let Self::Crafting { .. } = self {
                        String::from("your craft")
                    } else {
                        self.to_string().to_lowercase()
                    })
                    .send(self.severity_finishing(), false);
                None
            }
        }
    }

    fn handle_offset(
        &self,
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        player_pos: Pos,
        raw_nbor: Nbor,
    ) -> Option<PlannedAction> {
        if !matches!(*self, Self::Sleeping { .. }) {
            if let Err(failure) = envir.get_nbor(player_pos, raw_nbor) {
                message_writer.str(failure).send_error();
                return None;
            }
        }

        match &self {
            Self::Sleeping { .. }
            | Self::Peeking { .. }
            | Self::Dragging { .. }
            | Self::Pulping { .. } => {
                panic!("{self:?} {player_pos:?} {raw_nbor:?}");
            }
            Self::Crafting { .. }
            | Self::Waiting { .. }
            | Self::AutoTravel { .. }
            | Self::AutoDefend => {
                next_state.set(Self::Normal);
                None
            }
            Self::Normal => Some(PlannedAction::Step { to: raw_nbor }),
            Self::PickingNbor(picking_nbor) => {
                match picking_nbor {
                    PickingNbor::Attacking => {
                        next_state.set(Self::Normal);
                        Some(PlannedAction::Attack { target: raw_nbor })
                    }
                    PickingNbor::Smashing => {
                        next_state.set(Self::Normal);
                        Some(PlannedAction::Smash { target: raw_nbor })
                    }
                    PickingNbor::Pulping => {
                        //eprintln!("Inactive pulping");
                        if let Nbor::Horizontal(target) = raw_nbor {
                            //eprintln!("Activating pulping");
                            Some(PlannedAction::Pulp { target })
                        } else {
                            message_writer.you("can't pulp vertically").send_error();
                            None
                        }
                    }
                    PickingNbor::Peeking => {
                        Self::handle_peeking_offset(next_state, message_writer, raw_nbor)
                    }
                    PickingNbor::Closing => {
                        next_state.set(Self::Normal);
                        if let Nbor::Horizontal(target) = raw_nbor {
                            Some(PlannedAction::Close { target })
                        } else {
                            message_writer.you("can't close vertically").send_error();
                            None
                        }
                    }
                    PickingNbor::Dragging => {
                        next_state.set(Self::Dragging { from: player_pos });
                        Some(PlannedAction::Step { to: raw_nbor })
                    }
                    PickingNbor::Crafting { recipe_id } => {
                        if let Nbor::Horizontal(target) = raw_nbor {
                            // next_state is set when performing the action
                            Some(PlannedAction::StartCraft(StartCraft {
                                recipe_id: recipe_id.clone(),
                                target,
                            }))
                        } else {
                            message_writer.you("can't craft vertically").send_error();
                            None
                        }
                    }
                }
            }
        }
    }

    fn handle_peeking_offset(
        next_state: &mut NextState<Self>,
        message_writer: &mut MessageWriter,
        raw_nbor: Nbor,
    ) -> Option<PlannedAction> {
        match raw_nbor {
            Nbor::Up | Nbor::Down => {
                message_writer.you("can't peek vertically").send_error();
                None
            }
            Nbor::HERE => {
                message_writer.you("can't peek here").send_error();
                None
            }
            Nbor::Horizontal(
                HorizontalDirection::NorthWest
                | HorizontalDirection::SouthWest
                | HorizontalDirection::NorthEast
                | HorizontalDirection::SouthEast,
            ) => {
                message_writer.you("can't peek diagonally").send_error();
                None
            }
            Nbor::Horizontal(
                horizontal_direction @ (HorizontalDirection::North
                | HorizontalDirection::South
                | HorizontalDirection::East
                | HorizontalDirection::West),
            ) => {
                let target =
                    CardinalDirection::try_from(horizontal_direction).unwrap_or_else(|()| {
                        panic!("{horizontal_direction:?} should match a cardinal direction");
                    });
                //eprintln!("Activating peeking");
                // Not Self::Peeking, because that is not validated yet.
                next_state.set(Self::Normal);
                Some(PlannedAction::Peek { target })
            }
        }
    }

    fn handle_start_craft(
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        pos: Pos,
        recipe_id: ObjectId,
    ) -> Option<PlannedAction> {
        let craftable_nbors = envir
            .nbors_for_exploring(
                pos,
                QueuedInstruction::StartCraft {
                    recipe_id: recipe_id.clone(),
                },
            )
            .collect::<Vec<_>>();

        match craftable_nbors.len() {
            0 => {
                message_writer.str("no place to craft nearby").send_error();
                None
            }
            1 => {
                if let Nbor::Horizontal(horizontal_direction) =
                    craftable_nbors.first().expect("Single valid pos")
                {
                    // Craftig state is set when performing the action
                    Some(PlannedAction::StartCraft(StartCraft {
                        recipe_id,
                        target: *horizontal_direction,
                    }))
                } else {
                    message_writer.str("no place to craft nearby").send_error();
                    None
                }
            }
            _ => {
                next_state.set(Self::PickingNbor(PickingNbor::Crafting { recipe_id }));
                None
            }
        }
    }

    fn handle_attack(
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        pos: Pos,
    ) -> Option<PlannedAction> {
        let attackable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Attack)
            .collect::<Vec<_>>();
        match attackable_nbors.len() {
            0 => {
                message_writer.str("no targets nearby").send_error();
                None
            }
            1 => Some(PlannedAction::Attack {
                target: attackable_nbors[0],
            }),
            _ => {
                next_state.set(Self::PickingNbor(PickingNbor::Attacking));
                None
            }
        }
    }

    fn handle_smash(
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        pos: Pos,
    ) -> Option<PlannedAction> {
        let smashable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Smash)
            .collect::<Vec<_>>();
        match smashable_nbors.len() {
            0 => {
                message_writer.str("no targets nearby").send_error();
                None
            }
            1 => Some(PlannedAction::Smash {
                target: smashable_nbors[0],
            }),
            _ => {
                next_state.set(Self::PickingNbor(PickingNbor::Smashing));
                None
            }
        }
    }

    fn handle_pulp(
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        pos: Pos,
    ) -> Option<PlannedAction> {
        let pulpable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Pulp)
            .filter_map(|nbor| {
                if let Nbor::Horizontal(horizontal) = nbor {
                    Some(horizontal)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        //eprintln!("Pulping {} targets", pulpable_nbors.len());
        match pulpable_nbors.len() {
            0 => {
                message_writer.str("no targets nearby").send_error();
                None
            }
            1 => {
                //eprintln!("Pulping target found -> active");
                next_state.set(Self::Pulping {
                    direction: pulpable_nbors[0],
                });
                Some(PlannedAction::Pulp {
                    target: pulpable_nbors[0],
                })
            }
            _ => {
                //eprintln!("Pulping choice -> inactive");
                next_state.set(Self::PickingNbor(PickingNbor::Pulping));
                None
            }
        }
    }

    fn handle_close(
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        pos: Pos,
    ) -> Option<PlannedAction> {
        let closable_nbors = envir
            .nbors_for_exploring(pos, QueuedInstruction::Close)
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
                message_writer.str("nothing to close nearby").send_error();
                None
            }
            1 => Some(PlannedAction::Close {
                target: closable_nbors[0],
            }),
            _ => {
                next_state.set(Self::PickingNbor(PickingNbor::Closing));
                None
            }
        }
    }

    pub(crate) const fn color_in_progress(&self) -> Color {
        match self {
            Self::Normal | Self::PickingNbor(PickingNbor::Closing) => DEFAULT_TEXT_COLOR,
            Self::Waiting { .. }
            | Self::Sleeping { .. }
            | Self::PickingNbor { .. }
            | Self::Pulping { .. }
            | Self::Peeking { .. }
            | Self::Dragging { .. }
            | Self::Crafting { .. }
            | Self::AutoTravel { .. } => WARN_TEXT_COLOR,
            Self::AutoDefend => BAD_TEXT_COLOR,
        }
    }

    pub(crate) const fn severity_finishing(&self) -> Severity {
        match self {
            Self::Pulping { .. }
            | Self::Crafting { .. }
            | Self::PickingNbor(PickingNbor::Crafting { .. }) => Severity::Success,
            _ => Severity::Low,
        }
    }

    fn stop_peeking(
        next_state: &mut ResMut<NextState<Self>>,
        message_writer: &mut MessageWriter,
        envir: &Envir,
        player_pos: Pos,
        instruction: QueuedInstruction,
        now: Timestamp,
    ) -> Option<PlannedAction> {
        // Pretend to be Self::Normal
        Self::Normal
            .plan(
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
                        next_state.set(Self::Normal);
                    }
                },
            )
    }
}

impl fmt::Display for PlayerActionState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let picking_nbor_string;
        f.write_str(match self {
            Self::Normal => "",
            Self::PickingNbor(PickingNbor::Dragging { .. }) | Self::Dragging { .. } => "Dragging",
            Self::PickingNbor(picking_nbor) => {
                picking_nbor_string = String::from(match picking_nbor {
                    PickingNbor::Attacking => "Attacking",
                    PickingNbor::Smashing => "Smashing",
                    PickingNbor::Pulping => "Pulping",
                    PickingNbor::Peeking => "Peeking",
                    PickingNbor::Closing => "Closing",
                    PickingNbor::Dragging => unreachable!(),
                    PickingNbor::Crafting { .. } => "Crafting",
                }) + ": pick a direction";
                picking_nbor_string.as_str()
            }
            Self::Pulping { .. } => "Pulping",
            Self::Peeking { .. } => "Peeking",
            Self::Crafting { .. } => "Crafting",
            Self::Waiting { .. } => "Waiting",
            Self::Sleeping { .. } => "Sleeping",
            Self::AutoTravel { .. } => "Traveling",
            Self::AutoDefend => "Defending",
        })
    }
}

fn plan_auto_drag(
    envir: &Envir<'_, '_>,
    instruction_queue: &mut InstructionQueue,
    from: &Pos,
    enemy_name: Option<Fragment>,
) -> Option<PlannedAction> {
    if let Some(item) = envir.find_item(*from) {
        interrupt_on_danger(
            instruction_queue,
            enemy_name,
            PlannedAction::MoveItem {
                item,
                to: Nbor::HERE,
            },
        )
    } else {
        instruction_queue.interrupt(Interruption::Finished);
        None
    }
}

fn plan_auto_continue_craft(
    instruction_queue: &mut InstructionQueue,
    item: Entity,
    enemy_name: Option<Fragment>,
) -> Option<PlannedAction> {
    // Finishing a craft is checked when performing the action
    interrupt_on_danger(
        instruction_queue,
        enemy_name,
        PlannedAction::ContinueCraft { item },
    )
}

fn plan_auto_travel(
    envir: &Envir<'_, '_>,
    instruction_queue: &mut InstructionQueue,
    explored: &Explored,
    player: &ActorItem<'_>,
    target: Pos,
    enemy_name: Option<Fragment>,
) -> Option<PlannedAction> {
    if *player.pos == target {
        instruction_queue.interrupt(Interruption::Finished);
        None
    } else if let Some(enemy_name) = enemy_name {
        instruction_queue.interrupt(Interruption::Danger(enemy_name));
        None
    } else if player.stamina.breath() != Breath::Normal {
        instruction_queue.interrupt(Interruption::LowStamina);
        None
    } else {
        envir
            .path(
                *player.pos,
                target,
                Intelligence::Smart,
                |pos| explored.has_pos_been_seen(pos),
                player.speed(),
            )
            .map(|path| {
                envir
                    .nbor(*player.pos, path.first)
                    .expect("The first step should be a nbor")
            })
            .or_else(|| {
                // Full path not available
                envir
                    .nbors_for_moving(*player.pos, None, Intelligence::Smart, player.speed())
                    .map(|(nbor, nbor_pos, _)| (nbor, nbor_pos.vision_distance(target)))
                    .min_by_key(|(_, distance)| distance.as_tiles())
                    .filter(|(_, distance)| {
                        distance.in_range(VisionDistance::MAX_VISION_TILES as usize)
                    })
                    .map(|(nbor, _)| nbor)
            })
            .map(|to| PlannedAction::Step { to })
    }
}

fn plan_auto_defend(
    envir: &Envir<'_, '_>,
    instruction_queue: &mut InstructionQueue,
    player: &ActorItem<'_>,
    enemies: &[Pos],
) -> Option<PlannedAction> {
    if enemies.is_empty() {
        instruction_queue.interrupt(Interruption::Finished);
        None
    } else if player.stamina.breath() != Breath::Normal {
        instruction_queue.interrupt(Interruption::LowStamina);
        None
    } else if let Some(target) = enemies.iter().find_map(|pos| envir.nbor(*player.pos, *pos)) {
        Some(PlannedAction::Attack { target })
    } else {
        Some(PlannedAction::Stay)
    }
}

fn plan_auto_pulp(
    envir: &Envir<'_, '_>,
    instruction_queue: &mut InstructionQueue,
    player: &ActorItem<'_>,
    target: HorizontalDirection,
    enemy_name: Option<Fragment>,
) -> Option<PlannedAction> {
    //eprintln!("Post instruction pulp handling...");
    if envir
        .find_pulpable(player.pos.horizontal_nbor(target))
        .is_some()
    {
        interrupt_on_danger(
            instruction_queue,
            enemy_name,
            if player.stamina.breath() == Breath::Normal {
                //eprintln!("Keep pulping");
                PlannedAction::Pulp { target }
            } else {
                //eprintln!("Keep pulping after catching breath");
                PlannedAction::Stay
            },
        )
    } else {
        //eprintln!("Stop pulping");
        instruction_queue.interrupt(Interruption::Finished);
        None
    }
}

fn plan_auto_wait(
    instruction_queue: &mut InstructionQueue,
    now: Timestamp,
    until: &Timestamp,
    enemy_name: Option<Fragment>,
) -> Option<PlannedAction> {
    if *until <= now {
        instruction_queue.interrupt(Interruption::Finished);
        None
    } else {
        interrupt_on_danger(instruction_queue, enemy_name, PlannedAction::Stay)
    }
}

fn interrupt_on_danger(
    instruction_queue: &mut InstructionQueue,
    enemy_name: Option<Fragment>,
    planned_action: PlannedAction,
) -> Option<PlannedAction> {
    if let Some(enemy_name) = enemy_name {
        instruction_queue.interrupt(Interruption::Danger(enemy_name));
        None
    } else {
        Some(planned_action)
    }
}

fn plan_auto_sleep(
    instruction_queue: &mut InstructionQueue,
    now: Timestamp,
    until: &Timestamp,
) -> Option<PlannedAction> {
    // TODO interrupt on taking damage

    if *until <= now {
        instruction_queue.interrupt(Interruption::Finished);
        None
    } else {
        Some(PlannedAction::Sleep)
    }
}
