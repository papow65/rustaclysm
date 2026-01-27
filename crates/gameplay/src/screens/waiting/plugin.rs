use crate::GameplayScreenState;
use crate::screens::waiting::{
    WaitDuration, create_waiting_modal_key_bindings, create_waiting_modal_system, spawn_wait_modal,
};
use bevy::prelude::{App, In, IntoSystem as _, OnEnter, Plugin, Update};
use hud::manage_button_input;
use selection_list::selection_list_plugin;

pub(crate) struct WaitingModalPlugin;

impl Plugin for WaitingModalPlugin {
    fn build(&self, app: &mut App) {
        selection_list_plugin::<_, ()>(app, GameplayScreenState::Waiting, "select duration");

        app.add_systems(
            OnEnter(GameplayScreenState::Waiting),
            (
                create_waiting_modal_system.pipe(spawn_wait_modal),
                create_waiting_modal_key_bindings,
            ),
        );

        app.add_systems(Update, manage_button_input::<In<WaitDuration>>);
    }
}
