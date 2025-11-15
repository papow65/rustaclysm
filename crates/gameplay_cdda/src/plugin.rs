use crate::{Infos, TileLoader, regions::RegionsPlugin};
use application_state::ApplicationState;
use bevy::prelude::{App, Commands, OnExit, Plugin};
use gameplay_cdda_active_sav::ActiveSav;
use util::async_resource_plugin;

pub struct CddaPlugin;

impl Plugin for CddaPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            async_resource_plugin::<Infos>,
            async_resource_plugin::<TileLoader>,
            RegionsPlugin,
        ));

        // ActiveSav is created in the main menu
        app.add_systems(OnExit(ApplicationState::Gameplay), remove_active_sav);
    }
}

fn remove_active_sav(mut commands: Commands) {
    commands.remove_resource::<ActiveSav>();
}
