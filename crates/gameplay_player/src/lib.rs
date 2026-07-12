use application_state::ApplicationState;
use bevy::prelude::{Component, Entity, SubStates, TextColor};
use gameplay_crafting::RecipeSituation;
use gameplay_location::{CardinalDirection, HorizontalDirection, Pos};
use gameplay_log::{LogMessageTransience, Severity, Transient};
use hud::{BAD_TEXT_COLOR, HARD_TEXT_COLOR, WARN_TEXT_COLOR};
use std::fmt;
use units::Timestamp;

#[derive(Debug, Component)]
#[component(immutable)]
pub struct Player;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PickingNbor {
    Attacking,
    Smashing,
    Pulping,
    Peeking,
    Closing,
    Dragging,
    Crafting(RecipeSituation),
}

/// Current action of the player character
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, SubStates)]
#[source(ApplicationState = ApplicationState::Gameplay)]
pub enum PlayerActionState {
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
        from: Timestamp,
    },
    AutoTravel {
        target: Pos,
    },
    AutoDefend,
}

impl PlayerActionState {
    /// Automatic progress without movement or (expected) danger
    #[must_use]
    pub const fn is_still(&self) -> bool {
        matches!(
            *self,
            Self::Crafting { .. } | Self::Waiting { .. } | Self::Sleeping { .. }
        )
    }

    #[must_use]
    pub const fn is_automatic(&self) -> bool {
        !matches!(*self, Self::Normal | Self::PickingNbor(_))
    }

    #[must_use]
    pub const fn color_in_progress(&self) -> TextColor {
        match self {
            Self::Normal | Self::PickingNbor(PickingNbor::Closing) => HARD_TEXT_COLOR,
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

    #[must_use]
    pub const fn severity_finishing(&self) -> Severity {
        match self {
            Self::Pulping { .. }
            | Self::Crafting { .. }
            | Self::PickingNbor(PickingNbor::Crafting { .. }) => Severity::Success,
            _ => Severity::Neutral,
        }
    }
}

impl fmt::Display for PlayerActionState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let picking_nbor_string;
        f.write_str(match self {
            Self::Normal => "",
            Self::PickingNbor(PickingNbor::Dragging) | Self::Dragging { .. } => "Dragging",
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

impl LogMessageTransience for PlayerActionState {}
impl Transient for PlayerActionState {}
