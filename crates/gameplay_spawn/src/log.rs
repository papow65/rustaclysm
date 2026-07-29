use anyhow::Error as AnyError;
use bevy::prelude::{Entity, debug, error};

pub(super) fn log_spawn_result(spawned: Result<Entity, AnyError>) {
    match spawned {
        Ok(spawned) => debug!("Spawned {spawned:?}"),
        Err(error) => error!("Spawning skipped because of {error:#?}"),
    }
}
