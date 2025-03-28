use crate::ManualSection;
use crate::components::{LargeNode, ManualDisplay, ManualText};
use bevy::prelude::{
    Alpha as _, BackgroundColor, Changed, Commands, GlobalZIndex, Node, Query, RemovedComponents,
    Res, Text, Val, With, error,
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
    mut manual_background: Query<&mut BackgroundColor, With<ManualDisplay>>,
    mut manual_text: Query<&mut Text, With<ManualText>>,
    sections: Query<&ManualSection>,
    changed: Query<(), Changed<ManualSection>>,
    removed: RemovedComponents<ManualSection>,
) {
    if changed.is_empty() && removed.is_empty() {
        return;
    }

    let Ok(mut background_color) = manual_background
        .single_mut()
        .inspect_err(|error| error!("{error:?}"))
    else {
        return;
    };

    background_color.0.set_alpha(if large_nodes.is_empty() {
        PANEL_COLOR.0.alpha()
    } else {
        1.0
    });

    let mut sections = sections.iter().collect::<Vec<_>>();
    sections.sort_by_key(|section| section.sort_key());
    let Ok(mut manual_text) = manual_text
        .single_mut()
        .inspect_err(|error| error!("{error:?}"))
    else {
        return;
    };
    manual_text.0.replace_range(
        ..,
        sections
            .iter()
            .map(|section| section.text())
            .collect::<Vec<_>>()
            .join("\n")
            .as_str(),
    );
}
