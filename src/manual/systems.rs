use crate::gameplay::GameplayScreenState;
use crate::manual::ManualSection;
use crate::manual::components::{ManualDisplay, ManualText};
use bevy::prelude::{
    Alpha as _, BackgroundColor, BuildChildren as _, Changed, ChildBuild as _, Children, Commands,
    GlobalZIndex, Node, Query, RemovedComponents, Res, State, Text, Val, With,
};
use hud::{Fonts, PANEL_COLOR, SOFT_TEXT_COLOR, panel_node};

#[expect(clippy::needless_pass_by_value)]
pub(super) fn spawn_manual(mut commands: Commands, fonts: Res<Fonts>) {
    commands
        .spawn((
            Node {
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                ..panel_node()
            },
            PANEL_COLOR,
            GlobalZIndex(2),
            ManualDisplay,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::default(),
                SOFT_TEXT_COLOR,
                fonts.regular(),
                ManualText,
            ));
        });
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn update_manual(
    gameplay_screen_state: Option<Res<State<GameplayScreenState>>>,
    mut manual: Query<(&mut BackgroundColor, &mut Children), With<ManualDisplay>>,
    mut manual_text: Query<&mut Text, With<ManualText>>,
    sections: Query<&ManualSection>,
    changed: Query<(), Changed<ManualSection>>,
    removed: RemovedComponents<ManualSection>,
) {
    if changed.is_empty() && removed.is_empty() {
        return;
    }

    let mut manual = manual.single_mut();

    let background_color = &mut manual.0.0;
    background_color.set_alpha(
        if gameplay_screen_state
            .as_ref()
            .is_some_and(|state| state.get().large_node_bundle())
        {
            1.0
        } else {
            PANEL_COLOR.0.alpha()
        },
    );

    let mut sections = sections.iter().collect::<Vec<_>>();
    sections.sort_by_key(|section| section.sort_key());
    manual_text.single_mut().0.replace_range(
        ..,
        sections
            .iter()
            .map(|section| section.text())
            .collect::<Vec<_>>()
            .join("\n")
            .as_str(),
    );
}
