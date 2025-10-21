use bevy::prelude::{Local, World, debug};
use std::{env, sync::LazyLock};

#[derive(Default)]
pub(crate) struct ArchetypesOutput(Vec<String>);

pub(crate) fn log_archetypes(world: &mut World, mut last: Local<ArchetypesOutput>) {
    static ENABLED: LazyLock<bool> =
        LazyLock::new(|| env::var("LOG_ARCHETYPES") == Ok(String::from("1")));
    if !*ENABLED {
        return;
    }

    let output = world
        .archetypes()
        .iter()
        .filter(|archetype| !archetype.is_empty())
        .map(|archetype| {
            format!(
                "Archetype {} has {} entities, with components {}",
                archetype.id().index(),
                archetype.len(),
                archetype
                    .components()
                    .iter()
                    .map(
                        |component_id| world.components().get_name(*component_id).map_or_else(
                            || String::from("[unknown component]"),
                            |name| format!("{:?}", name.shortname())
                        )
                    )
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
        .collect::<Vec<_>>();

    if output != last.0 {
        for line in &output {
            debug!("{line}");
        }

        last.0 = output;
    }
}
