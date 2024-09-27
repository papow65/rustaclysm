use crate::gameplay::GameplayScreenState;
use crate::hud::{DefaultPanel, Fonts};
use crate::manual::components::ManualDisplay;
use bevy::prelude::{
    Alpha, BackgroundColor, BuildChildren, Children, Commands, NodeBundle, Query, Res, State, Text,
    TextBundle, TextSection, Val, With, ZIndex,
};

static GLOBAL_MANUAL_CONTENTS: &str = "\
    zoom ui         ctrl +/-\n\
    toggle this     F1\n\
    quit            ctrl c/q";

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
    open menu       esc\n\
    to main menu    F12\n";

static INVENTORY_MANUAL_CONTENTS: &str = "\
    select item     arrow up/down\n\
    set drop spot   numpad\n\
    drop item       d\n\
    examine item    e\n\
    take item       t\n\
    wield item      w\n\
    unwield item    u\n\
    close inventory i/esc\n\
    to main menu    F12\n";

static CRAFTING_MANUAL_CONTENTS: &str = "\
    select craft    arrow up/down\n\
    close crafting  &/esc\n\
    to main menu    F12\n";

static MENU_MANUAL_CONTENTS: &str = "\
    close this menu esc\n\
    to main menu    F12\n";

static DEATH_MANUAL_CONTENTS: &str = "\
    to main menu    F12/enter/space/esc\n";

#[expect(clippy::needless_pass_by_value)]
pub(super) fn spawn_manual(
    mut commands: Commands,
    default_panel: Res<DefaultPanel>,
    fonts: Res<Fonts>,
) {
    let mut background = default_panel.cloned();
    background.style.bottom = Val::Px(0.0);
    background.style.left = Val::Px(0.0);

    commands
        .spawn((
            NodeBundle {
                z_index: ZIndex::Global(2),
                ..background
            },
            ManualDisplay,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: String::from(BASE_MANUAL_CONTENTS),
                        style: fonts.standard(),
                    }],
                    ..Text::default()
                },
                ..TextBundle::default()
            });
        });
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn update_manual(
    gameplay_screen_state: Option<Res<State<GameplayScreenState>>>,
    default_panel: Res<DefaultPanel>,
    mut manual: Query<(&mut BackgroundColor, &mut Children), With<ManualDisplay>>,
    mut manual_text: Query<&mut Text>,
) {
    let mut manual = manual.single_mut();

    let background_color = &mut manual.0 .0;
    background_color.set_alpha(
        if gameplay_screen_state
            .as_ref()
            .is_some_and(|state| state.get().large_node_bundle())
        {
            1.0
        } else {
            default_panel.ref_().background_color.0.alpha()
        },
    );

    let manual_text_entity = manual
        .1
        .first()
        .expect("The manual should have a single child");
    let mut manual_text = manual_text
        .get_mut(*manual_text_entity)
        .expect("Text should exist for the manual");
    let manual_string = &mut manual_text
        .sections
        .get_mut(0)
        .expect("Manual text should have a single section")
        .value;

    manual_string.replace_range(
        ..,
        if let Some(gameplay_screen_state) = gameplay_screen_state {
            match gameplay_screen_state.get() {
                GameplayScreenState::Base => BASE_MANUAL_CONTENTS,
                GameplayScreenState::Inventory => INVENTORY_MANUAL_CONTENTS,
                GameplayScreenState::Crafting => CRAFTING_MANUAL_CONTENTS,
                GameplayScreenState::Menu => MENU_MANUAL_CONTENTS,
                GameplayScreenState::Death => DEATH_MANUAL_CONTENTS,
            }
        } else {
            ""
        },
    );
    *manual_string += GLOBAL_MANUAL_CONTENTS;
}
