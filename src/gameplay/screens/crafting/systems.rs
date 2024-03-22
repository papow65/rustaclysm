use super::resource::CraftingScreen;
use crate::prelude::*;
use bevy::prelude::*;
use std::ops::RangeInclusive;

#[allow(clippy::needless_pass_by_value)]
pub(super) fn spawn_crafting_screen(mut commands: Commands) {
    let panel = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    ..default()
                },
                ..default()
            },
            ScrollingList::default(),
        ))
        .id();
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            },
            StateBound::<GameplayScreenState>::default(),
        ))
        .with_children(|builder| {
            builder
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        justify_content: JustifyContent::Start,
                        margin: UiRect::horizontal(Val::Px(360.0)),
                        padding: UiRect::all(SMALL_SPACING),
                        overflow: Overflow::clip_y(),
                        ..default()
                    },
                    background_color: PANEL_COLOR.into(),
                    ..default()
                })
                .add_child(panel);
        });

    commands.insert_resource(CraftingScreen {
        panel,
        last_time: Timestamp::ZERO,
    });
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn clear_crafting_screen(
    clock: Clock,
    mut crafting_screen: ResMut<CraftingScreen>,
    children: Query<&Children>,
    mut styles: Query<&mut Style>,
) -> bool {
    if crafting_screen.last_time == clock.time() {
        return false;
    }
    crafting_screen.last_time = clock.time();

    if let Ok(children) = children.get(crafting_screen.panel) {
        for &child in children {
            if let Ok(mut style) = styles.get_mut(child) {
                style.display = Display::None;
            }
        }
    }

    true
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn update_crafting_screen(
    In(run): In<bool>,
    mut commands: Commands,
    location: Res<Location>,
    fonts: Res<Fonts>,
    infos: Res<Infos>,
    crafting_screen: Res<CraftingScreen>,
    players: Query<(&Pos, &BodyContainers), With<Player>>,
    items_and_furniture: Query<(&ObjectDefinition, &LastSeen, Option<&Parent>)>,
) {
    if !run {
        return;
    }

    let (&player_pos, body_containers) = players.single();

    let nearby = find_nearby(
        &location,
        &infos,
        &items_and_furniture,
        body_containers,
        player_pos,
    );

    commands
        .entity(crafting_screen.panel)
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                String::from("Nearby tools:"),
                fonts.regular(SOFT_TEXT_COLOR),
            ));

            let mut qualities = infos
                .qualities()
                .filter_map(|quality_id| {
                    find_quality(&nearby, quality_id).map(|amount| (quality_id, amount))
                })
                .map(|(quality_id, amount)| {
                    (
                        quality_id,
                        amount,
                        &infos
                            .quality(quality_id)
                            .expect("Quality should be found")
                            .name
                            .single,
                    )
                })
                .collect::<Vec<_>>();
            qualities.sort_by_key(|(.., quality)| *quality);
            for (_, amount, name) in qualities {
                parent.spawn(TextBundle::from_section(
                    format!("{amount} {name}"),
                    fonts.regular(DEFAULT_TEXT_COLOR),
                ));
            }
        });
}

const MAX_FIND_DISTANCE: i32 = 7;
const FIND_RANGE: RangeInclusive<i32> = (-MAX_FIND_DISTANCE)..=MAX_FIND_DISTANCE;

fn find_nearby<'a>(
    location: &'a Location,
    infos: &'a Infos,
    items_and_furniture: &'a Query<(&ObjectDefinition, &LastSeen, Option<&Parent>)>,
    body_containers: &'a BodyContainers,
    player_pos: Pos,
) -> Vec<&'a (ObjectId, i8)> {
    FIND_RANGE
        .flat_map(move |dz| {
            FIND_RANGE.flat_map(move |dx| {
                location
                    .all(player_pos.horizontal_offset(dx, dz))
                    .filter_map(|entity| items_and_furniture.get(*entity).ok())
                    .filter(|(_, last_seen, _)| **last_seen != LastSeen::Never)
            })
        })
        .chain(
            items_and_furniture
                .iter()
                .filter(|(.., parent)| parent.is_some_and(|p| p.get() == body_containers.hands))
                .chain(items_and_furniture.iter().filter(|(.., parent)| {
                    parent.is_some_and(|p| p.get() == body_containers.clothing)
                })),
        )
        .filter_map(|(definition, ..)| match definition.category {
            ObjectCategory::Item => infos.item(&definition.id).map(|item| &item.qualities),
            ObjectCategory::Furniture => infos
                .furniture(&definition.id)
                .and_then(|furniture| furniture.crafting_pseudo_item.as_ref())
                .and_then(|pseude_item| infos.item(pseude_item).map(|item| &item.qualities)),
            _ => None,
        })
        .flatten()
        .collect::<Vec<_>>()
}

fn find_quality(nearby: &[&(ObjectId, i8)], quality: &ObjectId) -> Option<i8> {
    nearby
        .iter()
        .filter(|item_quality| &item_quality.0 == quality)
        .map(|item_quality| item_quality.1)
        .max()
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn manage_crafting_keyboard_input(
    mut keys: Keys,
    mut next_gameplay_state: ResMut<NextState<GameplayScreenState>>,
    mut instruction_queue: ResMut<InstructionQueue>,
    mut crafting_screen: ResMut<CraftingScreen>,
) {
    for combo in keys.combos(Ctrl::Without) {
        match combo.key {
            Key::Code(KeyCode::Escape) | Key::Character('&')
                if combo.change == InputChange::JustPressed =>
            {
                next_gameplay_state.set(GameplayScreenState::Base);
            }
            _ => {}
        }
    }
}

/*#[allow(clippy::needless_pass_by_value)]
pub(super) fn manage_crafting_button_input(
    mut instruction_queue: ResMut<InstructionQueue>,
    interactions: Query<(&Interaction, &CraftingButton), (Changed<Interaction>, With<Button>)>,
    crafting_screen: Res<CraftingScreen>,
) {
    // TODO
}*/

#[allow(clippy::needless_pass_by_value)]
pub(super) fn remove_crafting_resource(mut commands: Commands) {
    commands.remove_resource::<CraftingScreen>();
}
