use crate::hud::{
    Fonts, BAD_TEXT_COLOR, GOOD_TEXT_COLOR, HARD_TEXT_COLOR, SOFT_TEXT_COLOR, WARN_TEXT_COLOR,
};
use bevy::prelude::{Color, Component, Entity, TextSpan, TextStyle};
use cdda_json_files::{ObjectId, Recipe};
use std::{cmp::Ordering, sync::Arc};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Component)]
pub(crate) struct RecipeSituation {
    pub(super) recipe_id: ObjectId,
    pub(super) name: Arc<str>,
    pub(super) autolearn: bool,
    pub(super) manuals: Vec<Arc<str>>,
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
                HARD_TEXT_COLOR
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

    pub(super) fn text_sections(
        &self,
        fonts: &Fonts,
        recipe: &Recipe,
    ) -> Vec<(TextSpan, TextStyle)> {
        let mut text_sections = vec![
            (TextSpan::new("Result: "), fonts.regular(SOFT_TEXT_COLOR)),
            (TextSpan::new(&*self.name), fonts.regular(self.color(true))),
            (
                TextSpan::new(format!("\n({})", self.recipe_id.fallback_name())),
                fonts.regular(SOFT_TEXT_COLOR),
            ),
        ];
        if let Some(skill_used) = &recipe.skill_used {
            text_sections.push((TextSpan::new("\n\nSkill: "), fonts.regular(SOFT_TEXT_COLOR)));
            text_sections.push((TextSpan::new(&**skill_used), fonts.regular(WARN_TEXT_COLOR)));
            text_sections.push((
                TextSpan::new("\nDifficulty: "),
                fonts.regular(SOFT_TEXT_COLOR),
            ));
            text_sections.push((
                TextSpan::new(format!("{}", recipe.difficulty)),
                fonts.regular(WARN_TEXT_COLOR),
            ));
        }
        if let Some(time) = &recipe.time {
            text_sections.push((
                TextSpan::new("\n\nDuration: "),
                fonts.regular(SOFT_TEXT_COLOR),
            ));
            text_sections.push((
                TextSpan::new(time.to_string()),
                fonts.regular(WARN_TEXT_COLOR),
            ));
        }
        if !self.qualities.is_empty() {
            text_sections.push((TextSpan::new("\n\nTools"), fonts.regular(SOFT_TEXT_COLOR)));
        }
        for quality in &self.qualities {
            text_sections.extend_from_slice(&quality.text_sections(fonts));
        }
        if !self.components.is_empty() {
            text_sections.push((
                TextSpan::new("\n\nComponents"),
                fonts.regular(SOFT_TEXT_COLOR),
            ));
        }
        for component in &self.components {
            text_sections.extend_from_slice(&component.text_sections(fonts));
        }
        text_sections.push((
            TextSpan::new("\n\nSource: "),
            fonts.regular(SOFT_TEXT_COLOR),
        ));
        text_sections.push((
            TextSpan::new(if self.autolearn { "Self-taught" } else { "" }),
            fonts.regular(GOOD_TEXT_COLOR),
        ));
        text_sections.push((
            TextSpan::new(if self.autolearn && !self.manuals.is_empty() {
                ", "
            } else {
                ""
            }),
            fonts.regular(GOOD_TEXT_COLOR),
        ));
        text_sections.push((
            TextSpan::new(self.manuals.join(", ")),
            fonts.regular(WARN_TEXT_COLOR),
        ));

        text_sections
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct QualitySituation {
    pub(super) name: Arc<str>,
    pub(super) present: Option<i8>,
    pub(super) required: u8,
}

impl QualitySituation {
    pub(super) fn is_present(&self) -> bool {
        self.present
            .is_some_and(|present| self.required as i8 <= present)
    }

    fn text_sections(&self, fonts: &Fonts) -> Vec<(TextSpan, TextStyle)> {
        let checked_style = fonts.regular(if self.is_present() {
            GOOD_TEXT_COLOR
        } else {
            BAD_TEXT_COLOR
        });
        let soft_style = fonts.regular(SOFT_TEXT_COLOR);

        let mut text_sections = vec![
            (TextSpan::new("\n- "), soft_style.clone()),
            (TextSpan::new(&*self.name), checked_style.clone()),
            (TextSpan::new(": a tool of "), soft_style.clone()),
        ];

        if let Some(present) = self.present {
            match present.cmp(&(self.required as i8)) {
                Ordering::Greater => {
                    text_sections.push((TextSpan::new(format!("{present}")), checked_style));
                    text_sections.push((
                        TextSpan::new(format!(" ({} required)", self.required)),
                        soft_style,
                    ));
                }
                Ordering::Equal => {
                    text_sections.push((TextSpan::new(format!("{present}")), checked_style));
                }
                Ordering::Less => {
                    text_sections.push((
                        TextSpan::new(format!("{} required", self.required)),
                        checked_style,
                    ));
                    text_sections
                        .push((TextSpan::new(format!(" ({present} present)")), soft_style));
                }
            }
        } else {
            text_sections.push((
                TextSpan::new(format!("{} required", self.required)),
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

    fn text_sections(&self, fonts: &Fonts) -> Vec<(TextSpan, TextStyle)> {
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
            text_sections.push((TextSpan::new(divider), soft_style.clone()));
            text_sections.extend_from_slice(&alternative.text_sections(fonts));
        }

        text_sections
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct AlternativeSituation {
    pub(super) id: ObjectId,
    pub(super) name: Arc<str>,
    pub(crate) required: u32,
    pub(super) present: u32,
    pub(crate) item_entities: Vec<Entity>,
}

impl AlternativeSituation {
    pub(super) const fn is_present(&self) -> bool {
        self.required <= self.present
    }

    fn text_sections(&self, fonts: &Fonts) -> Vec<(TextSpan, TextStyle)> {
        let checked_style = fonts.regular(if self.is_present() {
            GOOD_TEXT_COLOR
        } else {
            BAD_TEXT_COLOR
        });

        let mut text_sections = vec![(
            TextSpan::new(format!("{} {}", self.required, &self.name)),
            checked_style,
        )];

        if 0 < self.present {
            let only = if self.present < self.required {
                "only "
            } else {
                ""
            };

            text_sections.push((
                TextSpan::new(format!(" ({only}{} present)", self.present)),
                fonts.regular(SOFT_TEXT_COLOR),
            ));
        }

        text_sections
    }
}
