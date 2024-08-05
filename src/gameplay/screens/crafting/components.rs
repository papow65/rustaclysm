use crate::cdda::Recipe;
use crate::common::{
    Fonts, BAD_TEXT_COLOR, DEFAULT_TEXT_COLOR, GOOD_TEXT_COLOR, SOFT_TEXT_COLOR, WARN_TEXT_COLOR,
};
use crate::gameplay::ObjectId;
use bevy::prelude::{Color, Component, Entity, TextSection};
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Component)]
pub(crate) struct RecipeSituation {
    pub(super) recipe_id: ObjectId,
    pub(super) name: String,
    pub(super) autolearn: bool,
    pub(super) manuals: Vec<String>,
    pub(super) qualities: Vec<QualitySituation>,
    pub(super) components: Vec<ComponentSituation>,
}

impl RecipeSituation {
    pub(crate) const fn recipe_id(&self) -> &ObjectId {
        &self.recipe_id
    }

    pub(crate) fn consumed_components(&self) -> impl Iterator<Item = &AlternativeSituation> {
        self.components.iter().map(|component| {
            component
                .alternatives
                .iter()
                .find(|alternative| alternative.is_present())
                .expect("Crafting components should be present")
        })
    }

    pub(super) fn color(&self, selected: bool) -> Color {
        if self.craftable() {
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

    pub(super) fn craftable(&self) -> bool {
        self.qualities.iter().all(QualitySituation::is_present)
            && self.components.iter().all(ComponentSituation::is_present)
    }

    pub(super) fn text_sections(&self, fonts: &Fonts, recipe: &Recipe) -> Vec<TextSection> {
        let mut text_sections = vec![
            TextSection::new("Result: ", fonts.regular(SOFT_TEXT_COLOR)),
            TextSection::new(&self.name, fonts.regular(self.color(true))),
            TextSection::new(
                format!("\n({})", self.recipe_id.fallback_name()),
                fonts.regular(SOFT_TEXT_COLOR),
            ),
        ];
        if let Some(skill_used) = &recipe.skill_used {
            text_sections.push(TextSection::new(
                "\n\nSkill: ",
                fonts.regular(SOFT_TEXT_COLOR),
            ));
            text_sections.push(TextSection::new(skill_used, fonts.regular(WARN_TEXT_COLOR)));
            text_sections.push(TextSection::new(
                "\nDifficulty: ",
                fonts.regular(SOFT_TEXT_COLOR),
            ));
            text_sections.push(TextSection::new(
                format!("{}", recipe.difficulty),
                fonts.regular(WARN_TEXT_COLOR),
            ));
        }
        if let Some(time) = &recipe.time {
            text_sections.push(TextSection::new(
                "\n\nDuration: ",
                fonts.regular(SOFT_TEXT_COLOR),
            ));
            text_sections.push(TextSection::new(
                time.to_string(),
                fonts.regular(WARN_TEXT_COLOR),
            ));
        }
        if !self.qualities.is_empty() {
            text_sections.push(TextSection::new(
                "\n\nTools",
                fonts.regular(SOFT_TEXT_COLOR),
            ));
        }
        for quality in &self.qualities {
            text_sections.extend_from_slice(&quality.text_sections(fonts));
        }
        if !self.components.is_empty() {
            text_sections.push(TextSection::new(
                "\n\nComponents",
                fonts.regular(SOFT_TEXT_COLOR),
            ));
        }
        for component in &self.components {
            text_sections.extend_from_slice(&component.text_sections(fonts));
        }
        text_sections.push(TextSection::new(
            String::from("\n\nSource: "),
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct ComponentSituation {
    pub(super) alternatives: Vec<AlternativeSituation>,
}

impl ComponentSituation {
    pub(super) fn is_present(&self) -> bool {
        self.alternatives
            .iter()
            .any(AlternativeSituation::is_present)
    }

    fn text_sections(&self, fonts: &Fonts) -> Vec<TextSection> {
        let soft_style = fonts.regular(SOFT_TEXT_COLOR);

        let mut text_sections = Vec::new();

        for (index, alternative) in self.alternatives.iter().enumerate() {
            let divider = if index == 0 {
                "\n- "
            } else if index < self.alternatives.len() - 1 {
                ", "
            } else {
                ", or "
            };
            text_sections.push(TextSection::new(divider, soft_style.clone()));
            text_sections.extend_from_slice(&alternative.text_sections(fonts));
        }

        text_sections
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct AlternativeSituation {
    pub(super) id: ObjectId,
    pub(super) name: String,
    pub(crate) required: u32,
    pub(super) present: u32,
    pub(crate) item_entities: Vec<Entity>,
}

impl AlternativeSituation {
    pub(super) const fn is_present(&self) -> bool {
        self.required <= self.present
    }

    fn text_sections(&self, fonts: &Fonts) -> Vec<TextSection> {
        let checked_style = fonts.regular(if self.is_present() {
            GOOD_TEXT_COLOR
        } else {
            BAD_TEXT_COLOR
        });

        let mut text_sections = vec![TextSection::new(
            format!("{} {}", self.required, &self.name),
            checked_style.clone(),
        )];

        if 0 < self.present {
            let only = if self.present < self.required {
                "only "
            } else {
                ""
            };

            text_sections.push(TextSection::new(
                format!(" ({only}{} present)", self.present),
                fonts.regular(SOFT_TEXT_COLOR),
            ));
        }

        text_sections
    }
}
