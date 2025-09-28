//! This module provides the [`SidebarPlugin`]

mod components;
mod plugin;
mod systems;

pub(crate) use self::plugin::SidebarPlugin;

use self::components::{
    BreathText, DetailsText, EnemiesText, FpsText, HealthText, LastLogMessage, LastLogMessageCount,
    LogDisplay, PlayerActionStateText, SpeedTextSpan, StaminaText, TimeText, TransientLogMessage,
    WalkingModeTextSpan, WieldedText,
};
use self::systems::{spawn_sidebar, update_sidebar_systems, update_status_fps};
