use crate::{DebugText, DebugTextShown, Fonts, text::CheckedFont};
use bevy::prelude::{
    Commands, Entity, Or, Pickable, Query, Res, ResMut, Text, TextFont, TextSpan, With, Without,
};
use std::time::Instant;
use util::log_if_slow;

/// Set our font on all unchecked entities with [`Text`](`bevy::prelude::Text`) or [`TextSpan`](`bevy::prelude::TextSpan`)
#[expect(clippy::needless_pass_by_value)]
pub(super) fn add_missing_font(
    mut commands: Commands,
    fonts: Res<Fonts>,
    unchecked_texts: Query<
        (Entity, &TextFont),
        (Or<(With<Text>, With<TextSpan>)>, Without<CheckedFont>),
    >,
) {
    let start = Instant::now();

    //if !unchecked_texts.is_empty() {
    //    info!(
    //        "{} text entities where font is not yet checked",
    //        unchecked_texts.iter().len()
    //    );
    //}

    for (text_entity, text_font) in &unchecked_texts {
        let mut entity_commands = commands.entity(text_entity);
        if text_font.font == TextFont::default().font {
            entity_commands.insert((fonts.regular(), CheckedFont));
        } else {
            entity_commands.insert(CheckedFont);
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

    //if !texts_without_pickable.is_empty() {
    //    info!(
    //        "{} text entities where 'Pickable' is not yet set",
    //        texts_without_pickable.iter().len()
    //    );
    //}

    for text_entity in &texts_without_pickable {
        commands.entity(text_entity).insert(Pickable::IGNORE);
    }

    log_if_slow("add_missing_pickable", start);
}

/// Also updates the font size (0 or regular) for all entities with [`DebugText`]
#[expect(clippy::needless_pass_by_value)]
pub fn toggle_debug_text(
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
