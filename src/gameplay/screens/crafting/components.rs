use crate::prelude::{
    ObjectId, BAD_TEXT_COLOR, DEFAULT_TEXT_COLOR, GOOD_TEXT_COLOR, SOFT_TEXT_COLOR,
};
use bevy::prelude::{Color, Component};

#[derive(Component, Debug)]
pub(super) struct RecipeSituation {
    pub(super) recipe_id: ObjectId,
    pub(super) name: String,
    pub(super) autolearn: bool,
    pub(super) manuals: Vec<String>,
    pub(super) qualities_present: bool,
}

impl RecipeSituation {
    pub(super) const fn color(&self, selected: bool) -> Color {
        if self.qualities_present {
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
}
