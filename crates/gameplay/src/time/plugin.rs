use crate::Timeouts;
use application_state::ApplicationState;
use bevy::prelude::{App, Commands, OnEnter, OnExit, Plugin, Res};
use gameplay_cdda_active_sav::ActiveSav;
use units::Timestamp;

pub(crate) struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ApplicationState::Gameplay), create_timeouts);
        app.add_systems(OnExit(ApplicationState::Gameplay), remove_timeouts);
    }
}

/// Create resources that need other resources
#[expect(clippy::needless_pass_by_value)]
pub(crate) fn create_timeouts(mut commands: Commands, active_sav: Res<ActiveSav>) {
    const SEASON_LENGTH: u64 = 91; // TODO load from worldoptions.json

    let sav = active_sav.sav();
    let timestamp = Timestamp::new(sav.turn, SEASON_LENGTH);
    commands.insert_resource(Timeouts::new(timestamp));
}

fn remove_timeouts(mut commands: Commands) {
    commands.remove_resource::<Timeouts>();
}
