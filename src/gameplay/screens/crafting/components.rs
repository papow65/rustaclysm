use std::cmp::Ordering;

use crate::prelude::{
    Fonts, ObjectId, BAD_TEXT_COLOR, DEFAULT_TEXT_COLOR, GOOD_TEXT_COLOR, SOFT_TEXT_COLOR,
    WARN_TEXT_COLOR,
};
use bevy::prelude::{Color, Component, TextSection};

#[derive(Component, Debug)]
pub(super) struct RecipeSituation {
    pub(super) recipe_id: ObjectId,
    pub(super) name: String,
    pub(super) autolearn: bool,
    pub(super) manuals: Vec<String>,
    pub(super) qualities: Vec<QualitySituation>,
}

impl RecipeSituation {
    pub(super) fn color(&self, selected: bool) -> Color {
        if self.qualities.iter().all(QualitySituation::is_present) {
            if selected {
                GOOD_TEXT_COLOR
            } else {
                DEFAULT_TEXT_COLOR
            }
        } else if selected {
            BAD_TEXT_COLOR
        } else {
            SOFT_TEXT_COLOR
        }
    }

    pub(super) fn text_sections(&self, fonts: &Fonts) -> Vec<TextSection> {
        let mut text_sections = vec![
            TextSection::new("Result: ", fonts.regular(SOFT_TEXT_COLOR)),
            TextSection::new(&self.name, fonts.regular(self.color(true))),
            TextSection::new(
                format!("\n({})", self.recipe_id.fallback_name()),
                fonts.regular(SOFT_TEXT_COLOR),
            ),
        ];
        if !self.qualities.is_empty() {
            text_sections.push(TextSection::new(
                "\n\nTools",
                fonts.regular(SOFT_TEXT_COLOR),
            ));
        }
        for quality in &self.qualities {
            text_sections.extend_from_slice(&quality.text_sections(fonts));
        }
        text_sections.push(TextSection::new(
            String::from("\n\nSource:\n"),
            fonts.regular(SOFT_TEXT_COLOR),
        ));
        text_sections.push(TextSection::new(
            String::from(if self.autolearn { "Self-taught" } else { "" }),
            fonts.regular(GOOD_TEXT_COLOR),
        ));
        text_sections.push(TextSection::new(
            String::from(if self.autolearn && !self.manuals.is_empty() {
                ", "
            } else {
                ""
            }),
            fonts.regular(GOOD_TEXT_COLOR),
        ));
        text_sections.push(TextSection::new(
            self.manuals.join(", "),
            fonts.regular(WARN_TEXT_COLOR),
        ));

        text_sections
    }
}

#[derive(Debug)]
pub(super) struct QualitySituation {
    pub(super) name: String,
    pub(super) present: Option<i8>,
    pub(super) required: u8,
}

impl QualitySituation {
    pub(super) fn is_present(&self) -> bool {
        self.present
            .is_some_and(|present| self.required as i8 <= present)
    }

    fn text_sections(&self, fonts: &Fonts) -> Vec<TextSection> {
        let checked_style = fonts.regular(if self.is_present() {
            GOOD_TEXT_COLOR
        } else {
            BAD_TEXT_COLOR
        });
        let soft_style = fonts.regular(SOFT_TEXT_COLOR);

        let mut text_sections = vec![
            TextSection::new("\n- ", soft_style.clone()),
            TextSection::new(&self.name, checked_style.clone()),
            TextSection::new(": a tool of ", soft_style.clone()),
        ];

        if let Some(present) = self.present {
            match present.cmp(&(self.required as i8)) {
                Ordering::Greater => {
                    text_sections.push(TextSection::new(format!("{present}"), checked_style));
                    text_sections.push(TextSection::new(
                        format!(" ({} required)", self.required),
                        soft_style,
                    ));
                }
                Ordering::Equal => {
                    text_sections.push(TextSection::new(format!("{present}"), checked_style));
                }
                Ordering::Less => {
                    text_sections.push(TextSection::new(
                        format!("{} required", self.required),
                        checked_style,
                    ));
                    text_sections.push(TextSection::new(
                        format!(" ({present} present)"),
                        soft_style,
                    ));
                }
            }
        } else {
            text_sections.push(TextSection::new(
                format!("{} required", self.required),
                checked_style,
            ));
        }

        text_sections
    }
}
