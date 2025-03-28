use bevy::prelude::Component;
use std::sync::Arc;

/// When any entity exists with this component, the manual layout should adapt to it
#[derive(Debug, Component)]
#[component(immutable)]
pub struct LargeNode;

#[derive(Clone, Debug, Component)]
#[component(immutable)]
pub struct ManualSection {
    text: Arc<str>,
    sort_key: u8,
}

impl ManualSection {
    #[must_use]
    pub fn new(text: &[(&'static str, &'static str)], sort_key: u8) -> Self {
        Self {
            text: text
                .iter()
                .map(|(action, keys)| {
                    assert!(
                        action.chars().count() <= 15,
                        "Manual action too long: {action}"
                    );
                    assert!(keys.chars().count() <= 19, "Manual keys too long: {keys}");
                    format!("{action:<15} {keys}")
                })
                .collect::<Vec<_>>()
                .join("\n")
                .into(),
            sort_key,
        }
    }

    pub(super) fn text(&self) -> Arc<str> {
        self.text.clone()
    }

    pub(super) const fn sort_key(&self) -> u8 {
        self.sort_key
    }
}

#[derive(Debug, Component)]
#[component(immutable)]
pub(super) struct ManualDisplay;

#[derive(Debug, Component)]
#[component(immutable)]
pub(super) struct ManualText;
