use crate::gameplay::screens::{
    BaseScreenPlugin, CharacterScreenPlugin, CraftingScreenPlugin, DeathScreenPlugin,
    InventoryScreenPlugin, LoadingScreenPlugin, MenuScreenPlugin,
};
use bevy::prelude::{App, Plugin};

pub(crate) struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            BaseScreenPlugin,
            CharacterScreenPlugin,
            CraftingScreenPlugin,
            DeathScreenPlugin,
            InventoryScreenPlugin,
            LoadingScreenPlugin,
            MenuScreenPlugin,
        ));
    }
}
