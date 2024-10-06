use crate::gameplay::GameplayScreenState;
use crate::hud::{DefaultPanel, Fonts};
use crate::manual::components::{ManualDisplay, ManualText};
use crate::manual::ManualSection;
use bevy::prelude::{
    Alpha, BackgroundColor, BuildChildren, Changed, ChildBuild, Children, Commands, GlobalZIndex,
    Query, RemovedComponents, Res, State, Text, TextBundle, TextSection, Val, With,
};

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
        .spawn((background, GlobalZIndex(2), ManualDisplay))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: String::new(),
                            style: fonts.standard(),
                        }],
                        ..Text::default()
                    },
                    ..TextBundle::default()
                },
                ManualText,
            ));
        });
}

#[expect(clippy::needless_pass_by_value)]
pub(super) fn update_manual(
    gameplay_screen_state: Option<Res<State<GameplayScreenState>>>,
    default_panel: Res<DefaultPanel>,
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

    let mut sections = sections.iter().collect::<Vec<_>>();
    sections.sort_by_key(|section| section.sort_key());
    manual_text
        .single_mut()
        .sections
        .get_mut(0)
        .expect("Manual text should have a single section")
        .value
        .replace_range(
            ..,
            sections
                .iter()
                .map(|section| section.text())
                .collect::<Vec<_>>()
                .join("\n")
                .as_str(),
        );
}
