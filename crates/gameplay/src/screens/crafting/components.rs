use bevy::prelude::{Component, Entity, TextColor, TextFont, TextSpan};
use cdda_json_files::{
    CommonItemInfo, InfoId, Recipe, RequiredComponent, RequiredPart, RequiredTool,
};
use hud::{
    BAD_TEXT_COLOR, Fonts, GOOD_TEXT_COLOR, HARD_TEXT_COLOR, SOFT_TEXT_COLOR, WARN_TEXT_COLOR,
};
use std::{cmp::Ordering, num::NonZeroU32, sync::Arc};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Component)]
#[component(immutable)]
pub(crate) struct RecipeSituation {
    pub(super) recipe: Arc<Recipe>,
    pub(super) name: Arc<str>,
    pub(super) autolearn: bool,
    pub(super) manuals: Vec<Arc<str>>,
    pub(super) qualities: Vec<QualitySituation>,
    pub(super) tools: Vec<ToolSituation>,
    pub(super) components: Vec<ComponentSituation>,
}

impl RecipeSituation {
    pub(crate) const fn recipe(&self) -> &Arc<Recipe> {
        &self.recipe
    }

    /// Assumes being craftable
    pub(crate) fn consumed_tool_charges(&self) -> impl Iterator<Item = Consumed> {
        self.tools
            .iter()
            .filter_map(|tool| Self::consumed(&tool.alternatives))
    }

    /// Assumes being craftable
    pub(crate) fn consumed_components(&self) -> impl Iterator<Item = Consumed> {
        self.components
            .iter()
            .filter_map(|component| Self::consumed(&component.alternatives))
    }

    /// Assumes being craftable
    fn consumed<R: RequiredPart>(
        alternative_situations: &[AlternativeSituation<R>],
    ) -> Option<Consumed> {
        if alternative_situations
            .iter()
            .any(|alternative| match alternative.detected {
                DetectedQuantity::Missing => false,
                DetectedQuantity::Limited { .. } => !alternative.required.needs_quantity(),
                DetectedQuantity::Infinite => true,
            })
        {
            return None;
        }

        alternative_situations.iter().find_map(|alternative| {
            if let DetectedQuantity::Limited { from_entities, .. } = &alternative.detected {
                NonZeroU32::try_from(alternative.required.used_amount())
                    .ok()
                    .map(|amount| Consumed {
                        amount,
                        from_entities,
                    })
            } else {
                None
            }
        })
    }

    pub(super) fn color(&self, selected: bool) -> TextColor {
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
            && self.tools.iter().all(ToolSituation::is_present)
            && self.components.iter().all(ComponentSituation::is_present)
    }

    pub(super) fn text_sections(
        &self,
        fonts: &Fonts,
        recipe: &Arc<Recipe>,
    ) -> Vec<(TextSpan, TextColor, TextFont)> {
        let mut text_sections = vec![
            (TextSpan::new("Result: "), SOFT_TEXT_COLOR, fonts.regular()),
            (
                TextSpan::new(&*self.name),
                self.color(true),
                fonts.regular(),
            ),
            (
                TextSpan::new(format!("\n({})", self.recipe.id.fallback_name())),
                SOFT_TEXT_COLOR,
                fonts.regular(),
            ),
        ];
        if let Some(skill_used) = &recipe.skill_used {
            text_sections.push((
                TextSpan::new("\n\nSkill: "),
                SOFT_TEXT_COLOR,
                fonts.regular(),
            ));
            text_sections.push((
                TextSpan::new(&**skill_used),
                WARN_TEXT_COLOR,
                fonts.regular(),
            ));
            text_sections.push((
                TextSpan::new("\nDifficulty: "),
                SOFT_TEXT_COLOR,
                fonts.regular(),
            ));
            text_sections.push((
                TextSpan::new(format!("{}", recipe.difficulty)),
                WARN_TEXT_COLOR,
                fonts.regular(),
            ));
        }
        if let Some(time) = &recipe.time {
            text_sections.push((
                TextSpan::new("\n\nDuration: "),
                SOFT_TEXT_COLOR,
                fonts.regular(),
            ));
            text_sections.push((
                TextSpan::new(time.to_string()),
                WARN_TEXT_COLOR,
                fonts.regular(),
            ));
        }
        if !self.qualities.is_empty() || !self.tools.is_empty() {
            text_sections.push((TextSpan::new("\n\nTools"), SOFT_TEXT_COLOR, fonts.regular()));
        }
        for quality in &self.qualities {
            text_sections.extend_from_slice(&quality.text_sections(fonts));
        }
        for tool in &self.tools {
            text_sections.extend_from_slice(&tool.text_sections(fonts));
        }
        if !self.components.is_empty() {
            text_sections.push((
                TextSpan::new("\n\nComponents"),
                SOFT_TEXT_COLOR,
                fonts.regular(),
            ));
        }
        for component in &self.components {
            text_sections.extend_from_slice(&component.text_sections(fonts));
        }
        text_sections.push((
            TextSpan::new("\n\nSource: "),
            SOFT_TEXT_COLOR,
            fonts.regular(),
        ));
        text_sections.push((
            TextSpan::new(if self.autolearn { "Self-taught" } else { "" }),
            GOOD_TEXT_COLOR,
            fonts.regular(),
        ));
        text_sections.push((
            TextSpan::new(if self.autolearn && !self.manuals.is_empty() {
                ", "
            } else {
                ""
            }),
            GOOD_TEXT_COLOR,
            fonts.regular(),
        ));
        text_sections.push((
            TextSpan::new(self.manuals.join(", ")),
            WARN_TEXT_COLOR,
            fonts.regular(),
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

    fn text_sections(&self, fonts: &Fonts) -> Vec<(TextSpan, TextColor, TextFont)> {
        let checked_color = if self.is_present() {
            GOOD_TEXT_COLOR
        } else {
            BAD_TEXT_COLOR
        };

        let mut text_sections = vec![
            (TextSpan::new("\n- "), SOFT_TEXT_COLOR, fonts.regular()),
            (TextSpan::new(&*self.name), checked_color, fonts.regular()),
            (
                TextSpan::new(": a tool of "),
                SOFT_TEXT_COLOR,
                fonts.regular(),
            ),
        ];

        if let Some(present) = self.present {
            match present.cmp(&(self.required as i8)) {
                Ordering::Greater => {
                    text_sections.push((
                        TextSpan::new(format!("{present}")),
                        checked_color,
                        fonts.regular(),
                    ));
                    text_sections.push((
                        TextSpan::new(format!(" ({} required)", self.required)),
                        SOFT_TEXT_COLOR,
                        fonts.regular(),
                    ));
                }
                Ordering::Equal => {
                    text_sections.push((
                        TextSpan::new(format!("{present}")),
                        checked_color,
                        fonts.regular(),
                    ));
                }
                Ordering::Less => {
                    text_sections.push((
                        TextSpan::new(format!("{} required", self.required)),
                        checked_color,
                        fonts.regular(),
                    ));
                    text_sections.push((
                        TextSpan::new(format!(" ({present} present)")),
                        SOFT_TEXT_COLOR,
                        fonts.regular(),
                    ));
                }
            }
        } else {
            text_sections.push((
                TextSpan::new(format!("{} required", self.required)),
                checked_color,
                fonts.regular(),
            ));
        }

        text_sections
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct ToolSituation {
    pub(super) alternatives: Vec<AlternativeSituation<RequiredTool>>,
}

impl ToolSituation {
    pub(super) fn is_present(&self) -> bool {
        self.alternatives
            .iter()
            .any(AlternativeSituation::is_present)
    }

    fn text_sections(&self, fonts: &Fonts) -> Vec<(TextSpan, TextColor, TextFont)> {
        let mut text_sections = Vec::new();

        for (index, alternative) in self.alternatives.iter().enumerate() {
            let divider = if index == 0 {
                "\n- "
            } else if index < self.alternatives.len() - 1 {
                ", "
            } else {
                ", or "
            };
            text_sections.push((TextSpan::new(divider), SOFT_TEXT_COLOR, fonts.regular()));
            text_sections.extend_from_slice(&alternative.text_sections(fonts));
        }

        text_sections
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct ComponentSituation {
    pub(super) alternatives: Vec<AlternativeSituation<RequiredComponent>>,
}

impl ComponentSituation {
    pub(super) fn is_present(&self) -> bool {
        self.alternatives
            .iter()
            .any(AlternativeSituation::is_present)
    }

    fn text_sections(&self, fonts: &Fonts) -> Vec<(TextSpan, TextColor, TextFont)> {
        let mut text_sections = Vec::new();

        for (index, alternative) in self.alternatives.iter().enumerate() {
            let divider = if index == 0 {
                "\n- "
            } else if index < self.alternatives.len() - 1 {
                ", "
            } else {
                ", or "
            };
            text_sections.push((TextSpan::new(divider), SOFT_TEXT_COLOR, fonts.regular()));
            text_sections.extend_from_slice(&alternative.text_sections(fonts));
        }

        text_sections
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct AlternativeSituation<R: RequiredPart> {
    pub(super) id: InfoId<CommonItemInfo>,
    pub(super) name: Arc<str>,
    pub(super) required: R,
    pub(super) detected: DetectedQuantity<R>,
}

impl<R: RequiredPart> AlternativeSituation<R> {
    pub(super) fn is_present(&self) -> bool {
        match self.detected {
            DetectedQuantity::Missing => false,
            DetectedQuantity::Limited { present, .. } => self.required <= present,
            DetectedQuantity::Infinite => true,
        }
    }

    fn presence_color(&self) -> TextColor {
        if self.is_present() {
            GOOD_TEXT_COLOR
        } else {
            BAD_TEXT_COLOR
        }
    }

    fn text_sections(&self, fonts: &Fonts) -> Vec<(TextSpan, TextColor, TextFont)> {
        let mut text_sections = vec![(
            TextSpan::new(self.required.format(&self.name)),
            self.presence_color(),
            fonts.regular(),
        )];

        if self.required.needs_quantity() {
            text_sections.extend(
                (match self.detected {
                    DetectedQuantity::Missing => None,
                    DetectedQuantity::Limited { present, .. } => {
                        (0 < present.used_amount()).then(|| {
                            let only = if present < self.required { "only " } else { "" };
                            format!(" ({only}{} present)", present.used_amount())
                        })
                    }
                    DetectedQuantity::Infinite => Some(String::from(" (infinite)")),
                })
                .map(|details| (TextSpan::new(details), SOFT_TEXT_COLOR, fonts.regular())),
            );
        }

        text_sections
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum DetectedQuantity<R: RequiredPart> {
    Missing,

    // Note: `Consumed` should not be used here, because the purposes of present and amount differ.
    Limited {
        // We use `R` here, because for tools this can also be `RequiredTool::Uncharged`
        present: R,
        from_entities: Vec<Entity>,
    },

    Infinite,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Consumed<'a> {
    pub(crate) amount: NonZeroU32,
    pub(crate) from_entities: &'a [Entity],
}
