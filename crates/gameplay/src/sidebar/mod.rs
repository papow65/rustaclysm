//! This module provides the [`SidebarPlugin`]

mod components;
mod plugin;
mod systems;

pub(crate) use self::plugin::SidebarPlugin;

use self::components::{
    BreathText, DetailsText, EnemiesText, FpsText, HealthText, LastMessage, LastMessageCount,
    LogDisplay, PlayerActionStateText, SpeedTextSpan, StaminaText, TimeText, TransientMessage,
    WalkingModeTextSpan, WieldedText,
};
use self::systems::{spawn_sidebar, update_sidebar_systems, update_status_fps};
