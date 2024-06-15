use super::{components::ManualDisplay, resources::HudDefaults};
use crate::prelude::{ApplicationState, GameplayScreenState};
use bevy::prelude::{
    default, Alpha, BackgroundColor, BuildChildren, Children, Commands, NodeBundle, Query, Res,
    State, StateScoped, Text, TextBundle, TextSection, Val, With, ZIndex,
};

static BASE_MANUAL_CONTENTS: &str = "\
    move            numpad\n\
    move up/down    </>\n\
    attack npc      a\n\
    smash furniture s\n\
    pulp corpse     p\n\
    walking mode    +\n\
    auto defend     A\n\
    peek            tab\n\
    wait            |\n\
    sleep           $\n\
    show elevated   h\n\
    inventory       i\n\
    examine         x\n\
    examine map     X\n\
    auto travel     G\n\
    toggle map      m/M\n\
    camera angle    middle mouse button\n\
    reset angle     0\n\
    zoom            z/Z\n\
    zoom            scroll wheel\n\
    zoom ui         ctrl +/-\n\
    toggle this     F1\n\
    open menu       esc\n\
    to main menu    F12\n\
    quit            ctrl c/q";

static INVENTORY_MANUAL_CONTENTS: &str = "\
    select item     arrow up/down\n\
    set drop spot   numpad\n\
    drop item       d\n\
    examine item    e\n\
    take item       t\n\
    wield item      w\n\
    unwield item    u\n\
    zoom ui         ctrl +/-\n\
    toggle this     F1\n\
    close inventory &/esc\n\
    to main menu    F12\n\
    quit            ctrl c/q";

static CRAFTING_MANUAL_CONTENTS: &str = "\
    select craft    arrow up/down\n\
    zoom ui         ctrl +/-\n\
    toggle this     F1\n\
    close crafting  &/esc\n\
    to main menu    F12\n\
    quit            ctrl c/q";

static MENU_MANUAL_CONTENTS: &str = "\
    zoom ui         ctrl +/-\n\
    toggle this     F1\n\
    close this menu esc\n\
    to main menu    F12\n\
    quit            ctrl c/q";

static DEATH_MANUAL_CONTENTS: &str = "\
    zoom ui         ctrl +/-\n\
    toggle this     F1\n\
    to main menu    F12/enter/space/esc\n\
    quit            ctrl c/q";

#[allow(clippy::needless_pass_by_value)]
pub(super) fn spawn_manual(mut commands: Commands, hud_defaults: Res<HudDefaults>) {
    let mut background = hud_defaults.background.clone();
    background.style.bottom = Val::Px(0.0);
    background.style.left = Val::Px(0.0);

    commands
        .spawn((
            NodeBundle {
                z_index: ZIndex::Global(1),
                ..background
            },
            ManualDisplay,
            StateScoped(ApplicationState::Gameplay),
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: String::from(BASE_MANUAL_CONTENTS),
                        style: hud_defaults.text_style.clone(),
                    }],
                    ..default()
                },
                ..default()
            });
        });
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn update_manual(
    gameplay_screen_state: Res<State<GameplayScreenState>>,
    hud_defaults: Res<HudDefaults>,
    mut manual: Query<(&mut BackgroundColor, &mut Children), With<ManualDisplay>>,
    mut manual_text: Query<&mut Text>,
) {
    let mut manual = manual.single_mut();

    let background_color = &mut manual.0 .0;
    background_color.set_alpha(if gameplay_screen_state.get().large_node_bundle() {
        1.0
    } else {
        hud_defaults.background.background_color.0.alpha()
    });

    let manual_text_entity = manual
        .1
        .first()
        .expect("The manual should have a single child");
    manual_text
        .get_mut(*manual_text_entity)
        .expect("Text should exist for the manual")
        .sections
        .get_mut(0)
        .expect("Manual text should have a single section")
        .value
        .replace_range(
            ..,
            match gameplay_screen_state.get() {
                GameplayScreenState::Base => BASE_MANUAL_CONTENTS,
                GameplayScreenState::Inventory => INVENTORY_MANUAL_CONTENTS,
                GameplayScreenState::Crafting => CRAFTING_MANUAL_CONTENTS,
                GameplayScreenState::Menu => MENU_MANUAL_CONTENTS,
                GameplayScreenState::Death => DEATH_MANUAL_CONTENTS,
                GameplayScreenState::Inapplicable => {
                    panic!("Gameplay manual contents should not be updated outside of gameplay")
                }
            },
        );
}
