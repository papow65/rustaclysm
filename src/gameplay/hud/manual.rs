use super::{components::ManualDisplay, resources::HudDefaults};
use crate::prelude::{ApplicationState, StateBound};
use bevy::prelude::{default, BuildChildren, Commands, Res, Text, TextBundle, TextSection, Val};

#[allow(clippy::needless_pass_by_value)]
pub(super) fn spawn_manual(mut commands: Commands, hud_defaults: Res<HudDefaults>) {
    let mut background = hud_defaults.background.clone();
    background.style.bottom = Val::Px(0.0);
    background.style.left = Val::Px(0.0);

    commands
        .spawn((
            background,
            ManualDisplay,
            StateBound::<ApplicationState>::default(),
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: String::new()
                            + "move            numpad\n"
                            + "move up/down    </>\n"
                            + "attack npc      a\n"
                            + "smash furniture s\n"
                            + "pulp corpse     p\n"
                            + "walking mode    +\n"
                            + "auto defend     tab\n"
                            + "wait            |\n"
                            + "sleep           $\n"
                            + "show elevated   h\n"
                            + "inventory       i\n"
                            + "examine         x\n"
                            + "examine map     X\n"
                            + "auto travel     G\n"
                            + "toggle map      m/M\n"
                            + "camera angle    middle mouse button\n"
                            + "reset angle     0\n"
                            + "zoom            z/Z\n"
                            + "zoom            scroll wheel\n"
                            + "zoom ui         ctrl +/-\n"
                            + "toggle this     F1\n"
                            + "menu            esc\n"
                            + "main menu       F12\n"
                            + "quit            ctrl c/q",
                        style: hud_defaults.text_style.clone(),
                    }],
                    ..default()
                },
                ..default()
            });
        });
}
