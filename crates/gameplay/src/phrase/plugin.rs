use application_state::ApplicationState;
use bevy::prelude::{App, DespawnOnExit, Local, OnEnter, Plugin, World};
use hud::toggle_debug_text;
use keyboard::KeyBindings;
use manual::ManualSection;
use std::time::Instant;
use util::log_if_slow;

pub(crate) struct PhrasePlugin;

impl Plugin for PhrasePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(ApplicationState::Gameplay),
            create_phrase_key_bindings,
        );
    }
}

#[expect(clippy::needless_pass_by_value)]
fn create_phrase_key_bindings(
    world: &mut World,
    bindings: Local<KeyBindings<ApplicationState, (), ()>>,
) {
    let start = Instant::now();

    bindings.spawn(world, ApplicationState::Gameplay, |bindings| {
        bindings.add('D', toggle_debug_text);
    });

    world.spawn((
        ManualSection::new(&[("show cdda ids", "D")], 200),
        DespawnOnExit(ApplicationState::Gameplay),
    ));

    log_if_slow("create_phrase_key_bindings", start);
}
