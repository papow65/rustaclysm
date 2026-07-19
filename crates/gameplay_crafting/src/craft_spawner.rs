use bevy::prelude::Entity;
use cdda_json_files::Recipe;
use gameplay_cdda::Error;
use gameplay_location::Pos;
use std::sync::Arc;

pub trait CraftSpawner {
    fn spawn_craft(&mut self, pos: Pos, recipe: Arc<Recipe>) -> Result<Entity, Error>;
}
