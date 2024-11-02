use crate::application::ApplicationState;
use crate::gameplay::cdda::{region_assets::RegionAssetsPlugin, ActiveSav, Infos, TileLoader};
use crate::util::AsyncResourcePlugin;
use bevy::prelude::{App, Commands, OnExit, Plugin};

pub(crate) struct CddaPlugin;

impl Plugin for CddaPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AsyncResourcePlugin::<Infos>::default(),
            AsyncResourcePlugin::<TileLoader>::default(),
            RegionAssetsPlugin,
        ));

        // ActiveSav is created in the main menu
        app.add_systems(OnExit(ApplicationState::Gameplay), remove_active_sav);
    }
}

fn remove_active_sav(mut commands: Commands) {
    commands.remove_resource::<ActiveSav>();
}
