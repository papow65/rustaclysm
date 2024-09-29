use bevy::prelude::Component;
use std::sync::Arc;

#[derive(Clone, Debug, Component)]
pub(crate) struct ManualSection {
    text: Arc<str>,
    sort_key: u8,
}

impl ManualSection {
    pub(crate) fn new(text: &[(&'static str, &'static str)], sort_key: u8) -> Self {
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
pub(super) struct ManualDisplay;

#[derive(Debug, Component)]
pub(super) struct ManualText;
