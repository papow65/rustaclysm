use crate::gameplay::{DebugText, DebugTextShown};
use crate::{application::ApplicationState, hud::Fonts};
use crate::{keyboard::KeyBindings, manual::ManualSection, util::log_if_slow};
use bevy::prelude::{App, Local, OnEnter, Plugin, Query, Res, ResMut, TextFont, With, World};
use std::time::Instant;

pub(crate) struct PhrasePlugin;

impl Plugin for PhrasePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DebugTextShown::default());

        app.add_systems(
            OnEnter(ApplicationState::Gameplay),
            create_phrase_key_bindings,
        );
    }
}

#[allow(clippy::needless_pass_by_value)]
fn create_phrase_key_bindings(
    world: &mut World,
    bindings: Local<KeyBindings<ApplicationState, (), ()>>,
) {
    let start = Instant::now();

    bindings.spawn(
        world,
        ApplicationState::Gameplay,
        |bindings| {
            bindings.add('D', toggle_debug_text);
        },
        ManualSection::new(&[("show cdda ids", "D")], 200),
    );

    log_if_slow("create_phrase_key_bindings", start);
}

#[allow(clippy::needless_pass_by_value)]
fn toggle_debug_text(
    fonts: Res<Fonts>,
    mut shown: ResMut<DebugTextShown>,
    mut debug_fonts: Query<&mut TextFont, With<DebugText>>,
) {
    shown.0 = !shown.0;

    let size = shown.text_font(fonts.regular()).font_size;
    for mut font in &mut debug_fonts {
        font.font_size = size;
    }
}