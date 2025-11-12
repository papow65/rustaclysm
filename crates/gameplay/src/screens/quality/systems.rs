use crate::screens::{find_nearby, find_nearby_pseudo, nearby_qualities};
use crate::{BodyContainers, GameplayScreenState, Item, Player, Shared};
use bevy::picking::Pickable;
use bevy::prelude::{
    AnyOf, Commands, DespawnOnExit, KeyCode, Local, NextState, Query, Res, ResMut, Single, Text,
    With, World,
};
use cdda_json_files::{FurnitureInfo, TerrainInfo};
use gameplay_location::{LocationCache, Pos};
use gameplay_model::LastSeen;
use hud::{Fonts, GOOD_TEXT_COLOR, WARN_TEXT_COLOR, scroll_screen};
use keyboard::KeyBindings;
use manual::ManualSection;
use std::time::Instant;
use util::{log_if_slow, uppercase_first};

#[expect(clippy::needless_pass_by_value)]
pub(super) fn spawn_crafting_screen(
    mut commands: Commands,
    location: Res<LocationCache>,
    fonts: Res<Fonts>,
    player: Single<(&Pos, &BodyContainers), With<Player>>,
    items: Query<(Item, &LastSeen)>,
    infrastructure: Query<(
        AnyOf<(&Shared<FurnitureInfo>, &Shared<TerrainInfo>)>,
        &LastSeen,
    )>,
) {
    let qualitiy_list = scroll_screen(&mut commands, GameplayScreenState::Quality);

    let (&player_pos, body_containers) = *player;

    let nearby_items = find_nearby(&location, &items, player_pos, body_containers);
    let nearby_pseudo_items = find_nearby_pseudo(&location, &infrastructure, player_pos);
    let nearby_qualities = nearby_qualities(&nearby_items, &nearby_pseudo_items);
    //trace!("{:?}", &nearby_manuals);

    let mut shown_qualities = nearby_qualities
        .iter()
        .map(|(quality, amount)| (amount, uppercase_first(quality.name.single.clone())))
        .collect::<Vec<_>>();
    shown_qualities.sort_by_key(|(.., name)| name.clone());

    commands.entity(qualitiy_list).with_children(|parent| {
        parent.spawn((
            Text::from("Nearby qualities:"),
            WARN_TEXT_COLOR,
            fonts.regular(),
            Pickable::IGNORE,
        ));

        for (amount, name) in shown_qualities {
            parent.spawn((
                Text::from(format!("{amount} {name}")),
                GOOD_TEXT_COLOR,
                fonts.regular(),
                Pickable::IGNORE,
            ));
        }
    });
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn create_crafting_key_bindings(
    world: &mut World,
    fresh_bindings: Local<KeyBindings<GameplayScreenState, (), ()>>,
) {
    let start = Instant::now();

    fresh_bindings.spawn(world, GameplayScreenState::Quality, |bindings| {
        bindings.add(KeyCode::Escape, exit_qualities);
        bindings.add('q', exit_qualities);
    });

    world.spawn((
        ManualSection::new(&[("close qualities", "esc/q")], 100),
        DespawnOnExit(GameplayScreenState::Quality),
    ));

    log_if_slow("create_crafting_key_bindings", start);
}

fn exit_qualities(mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>) {
    let start = Instant::now();

    next_gameplay_state.set(GameplayScreenState::Base);

    log_if_slow("exit_qualities", start);
}
