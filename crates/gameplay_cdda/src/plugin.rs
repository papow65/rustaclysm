use crate::{Exploration, Infos, TileLoader, region_assets::RegionAssetsPlugin};
use application_state::ApplicationState;
use bevy::prelude::{App, Commands, OnExit, Plugin, StateScopedEventsAppExt as _};
use gameplay_cdda_active_sav::ActiveSav;
use util::AsyncResourcePlugin;

pub struct CddaPlugin;

impl Plugin for CddaPlugin {
    fn build(&self, app: &mut App) {
        app.add_state_scoped_event::<Exploration>(ApplicationState::Gameplay);

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
