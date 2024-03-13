use crate::prelude::*;
use bevy::prelude::*;

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn spawn_manual(mut commands: Commands, hud_defaults: Res<HudDefaults>) {
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
                            + "move          numpad\n"
                            + "up/down       </>\n"
                            + "attack        a\n"
                            + "smash         s\n"
                            + "walking mode  +\n"
                            + "auto defend   tab\n"
                            + "wait          |\n"
                            + "sleep         $\n"
                            + "show elevated h\n"
                            + "inventory     i\n"
                            + "examine       x\n"
                            + "examine map   X\n"
                            + "toggle map    m/M\n"
                            + "camera angle  hold middle\n"
                            + "reset angle   0\n"
                            + "zoom          z/Z\n"
                            + "zoom          scroll wheel\n"
                            + "zoom ui       ctrl +/-\n"
                            + "toggle this   f1\n"
                            + "menu          esc\n"
                            + "main menu     f12\n"
                            + "quit          ctrl c/q",
                        style: hud_defaults.text_style.clone(),
                    }],
                    ..Text::default()
                },
                ..default()
            });
        });
}
