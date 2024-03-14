/*! This plugin defines the [`HudPlugin`] */

mod components;
mod input;
mod manual;
mod plugin;
mod resources;
mod sidebar;

pub(crate) use self::plugin::HudPlugin;
use self::{
    components::{LogDisplay, ManualDisplay, StatusDisplay},
    input::manage_hud_keyboard_input,
    manual::spawn_manual,
    resources::{despawn_hud_resources, spawn_hud_resources, HudDefaults, StatusTextSections},
    sidebar::{
        spawn_sidebar, update_log, update_status_detais, update_status_enemies, update_status_fps,
        update_status_health, update_status_player_action_state, update_status_player_wielded,
        update_status_speed, update_status_stamina, update_status_time,
    },
};
