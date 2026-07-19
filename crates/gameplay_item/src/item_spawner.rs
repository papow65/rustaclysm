use bevy::{ecs::relationship::Relationship, prelude::Entity};
use cdda_json_files::CddaItem;
use gameplay_cdda::Error;
use gameplay_location::Pos;

use crate::Amount;

pub trait ItemSpawner {
    fn spawn_item<R: Relationship>(
        &mut self,
        parent: R,
        pos: Option<Pos>,
        item: &CddaItem,
        amount: Amount,
    ) -> Result<Entity, Error>;
}
