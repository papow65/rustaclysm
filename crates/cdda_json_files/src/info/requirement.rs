use crate::{
    Alternative, ComponentPresence, Error, Ignored, InfoId, Recipe, RequiredPresence,
    RequiredQualities, ToolPresence, Using,
};
use serde::Deserialize;
use std::iter::once;

#[derive(Debug, Deserialize)]
pub struct Requirement {
    pub id: InfoId<Requirement>,

    #[serde(default)]
    pub qualities: RequiredQualities,

    #[serde(default)]
    pub components: Vec<Vec<Alternative<ComponentPresence>>>,

    #[serde(default)]
    pub tools: Vec<Vec<Alternative<ToolPresence>>>,

    #[serde(flatten)]
    _ignored: Ignored<Self>,
}

#[derive(Debug)]
pub struct CalculatedRequirement {
    pub qualities: RequiredQualities,
    pub components: Vec<Vec<Alternative<ComponentPresence>>>,
    pub tools: Vec<Vec<Alternative<ToolPresence>>>,
}

impl CalculatedRequirement {
    fn combine(self, others: Vec<Self>) -> Self {
        let (other_qualities, other_components_and_tools): (Vec<_>, Vec<_>) = others
            .into_iter()
            .map(|other| (other.qualities, (other.components, other.tools)))
            .unzip();
        let (other_components, other_tools): (Vec<_>, Vec<_>) = other_components_and_tools
            .into_iter()
            .map(|other| (other.0, other.1))
            .unzip();

        Self {
            qualities: RequiredQualities(
                once(self.qualities.0)
                    .chain(other_qualities.into_iter().map(|q| q.0))
                    .flatten()
                    .collect(),
            ),
            components: once(self.components)
                .chain(other_components)
                .flatten()
                .collect(),
            tools: once(self.tools).chain(other_tools).flatten().collect(),
        }
    }
}

impl From<&Requirement> for CalculatedRequirement {
    fn from(requirement: &Requirement) -> Self {
        Self {
            qualities: requirement.qualities.clone(),
            components: clone(&requirement.components, 1),
            tools: clone(&requirement.tools, 1),
        }
    }
}

impl TryFrom<&Using> for CalculatedRequirement {
    type Error = Error;
    fn try_from(using: &Using) -> Result<Self, Error> {
        let requirement = using.requirement.get()?;
        Ok(Self {
            qualities: requirement.qualities.clone(),
            components: clone(&requirement.components, using.factor),
            tools: clone(&requirement.tools, using.factor),
        })
    }
}

impl TryFrom<&Recipe> for CalculatedRequirement {
    type Error = Error;
    fn try_from(recipe: &Recipe) -> Result<Self, Error> {
        Ok(Self {
            qualities: recipe.qualities.clone(),
            components: clone(&recipe.components, 1),
            tools: clone(&recipe.tools, 1),
        }
        .combine(
            recipe
                .using
                .iter()
                .map(Self::try_from)
                .collect::<Result<Vec<_>, Error>>()?,
        ))
    }
}

fn clone<R: RequiredPresence>(
    alternatives: &[Vec<Alternative<R>>],
    factor: u32,
) -> Vec<Vec<Alternative<R>>> {
    alternatives
        .iter()
        .map(|v| v.iter().map(|a| a * R::from(factor)).collect())
        .collect()
}
