use crate::gameplay::cdda::Error;
use bevy::prelude::{Entity, debug, error};

pub(super) fn log_spawn_result(spawned: Result<Entity, Error>) {
    match spawned {
        Ok(spawned) => debug!("Spawned {spawned:?}"),
        Err(error) => error!("Spawning skipped because of {error:#?}"),
    }
}
