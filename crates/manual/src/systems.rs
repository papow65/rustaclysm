use crate::ManualSection;
use crate::components::{LargeNode, ManualDisplay, ManualText};
use bevy::prelude::{
    Alpha as _, BackgroundColor, BuildChildren as _, Changed, ChildBuild as _, Children, Commands,
    GlobalZIndex, Node, Query, RemovedComponents, Res, Text, Val, With,
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
    large_nodes: Query<(), With<LargeNode>>,
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
    background_color.set_alpha(if large_nodes.is_empty() {
        PANEL_COLOR.0.alpha()
    } else {
        1.0
    });

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
