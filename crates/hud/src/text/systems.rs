use crate::text::{DebugText, DebugTextShown, FiraFont, Fonts};
use bevy::prelude::{
    Added, AssetEvent, AssetId, Assets, Commands, Entity, Font, MessageReader, Or, Pickable, Query,
    Res, ResMut, Text, TextFont, TextSpan, With, Without, debug,
};
use std::time::Instant;
use util::log_if_slow;

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn setup_global_font(
    mut font_events: MessageReader<AssetEvent<Font>>,
    fira: Res<FiraFont>,
    mut font_assets: ResMut<Assets<Font>>,
    mut text_fonts: Query<&mut TextFont>,
) {
    for font_event in font_events.read() {
        if let AssetEvent::Added { id } = font_event {
            // Future TextFont components: set TextFont::default().font
            let font = font_assets
                .get(*id)
                .expect("Added font should be found")
                .clone();
            font_assets
                .insert(AssetId::default(), font.clone())
                .expect("Font asset should have a valid generation");

            // Existing TextFont components: replace the font(source)
            for mut text_font in &mut text_fonts {
                text_font.font = fira.font_source.clone();
            }
        }
    }
}

/// Set our font on all unchecked entities with [`Text`](`bevy::prelude::Text`) or [`TextSpan`](`bevy::prelude::TextSpan`)
pub(super) fn set_default_font_size(
    mut unchecked_texts: Query<&mut TextFont, Or<(Added<Text>, Added<TextSpan>)>>,
) {
    let start = Instant::now();

    for mut text_font in &mut unchecked_texts {
        if text_font.font_size == TextFont::default().font_size {
            text_font.font_size = Fonts::regular().font_size;
        }
    }

    log_if_slow("add_missing_font", start);
}

/// Make all entities with [`Text`](`bevy::prelude::Text`) or [`TextSpan`](`bevy::prelude::TextSpan`) unpickable, unless [`Pickable`](`bevy::prelude::Pickable`) is already set
pub(super) fn add_missing_pickable(
    mut commands: Commands,
    texts_without_pickable: Query<Entity, (Or<(With<Text>, With<TextSpan>)>, Without<Pickable>)>,
) {
    let start = Instant::now();

    for text_entity in &texts_without_pickable {
        commands.entity(text_entity).insert(Pickable::IGNORE);
    }

    log_if_slow("add_missing_pickable", start);
}

/// Also updates the font size (0 or regular) for all entities with [`DebugText`]
pub fn toggle_debug_text(
    mut shown: ResMut<DebugTextShown>,
    mut debug_fonts: Query<&mut TextFont, With<DebugText>>,
) {
    shown.0 = !shown.0;

    debug!("Debug: {shown:?}");

    let size = shown.text_font(Fonts::regular()).font_size;
    for mut font in &mut debug_fonts {
        font.font_size = size;
    }
}
