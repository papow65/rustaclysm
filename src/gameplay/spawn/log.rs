use crate::gameplay::cdda::Error;
use bevy::ecs::entity::Entity;

pub(super) fn log_spawn_result(spawned: Result<Entity, Error>) {
    match spawned {
        Ok(spawned) => println!("Spawned {spawned:?}"),
        Err(error) => eprintln!("Spawning skipped because of {error:#?}"),
    }
}
