use crate::{GameplayScreenState, Infos, Player, Pos, spawn::TileSpawner};
use application_state::ApplicationState;
use bevy::prelude::{
    KeyCode, Local, NextState, Res, ResMut, Single, StateScoped, With, World, debug,
};
use keyboard::KeyBindings;
use manual::ManualSection;
use std::time::Instant;
use util::log_if_slow;

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn create_gameplay_key_bindings(
    world: &mut World,
    bindings: Local<KeyBindings<ApplicationState, (), ()>>,
) {
    let start = Instant::now();

    bindings.spawn(world, ApplicationState::Gameplay, |bindings| {
        bindings.add('!', spawn_zombies);
        bindings.add(KeyCode::F12, to_main_menu);
    });

    world.spawn((
        ManualSection::new(
            &[("add debug zeds", "!"), ("to main menu", "F12")],
            u8::MAX - 2,
        ),
        StateScoped(ApplicationState::Gameplay),
    ));

    log_if_slow("create_gameplay_key_bindings", start);
}

#[expect(clippy::needless_pass_by_value)]
fn spawn_zombies(
    mut tile_spawner: TileSpawner,
    infos: Res<Infos>,
    player: Option<Single<&Pos, With<Player>>>,
) {
    if let Some(player_pos) = player {
        tile_spawner.spawn_zombies(&infos, **player_pos);
    }
}

fn to_main_menu(mut next_application_state: ResMut<NextState<GameplayScreenState>>) {
    debug!("Unloading");
    next_application_state.set(GameplayScreenState::Unloading);
}
