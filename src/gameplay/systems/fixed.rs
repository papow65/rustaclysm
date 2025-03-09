use crate::gameplay::{
    MapAsset, MapMemoryAsset, OvermapAsset, OvermapBufferAsset, Pos, SubzoneLevel, ZoneLevel,
};
use crate::util::log_if_slow;
use bevy::asset::UntypedAssetLoadFailedEvent;
use bevy::prelude::{
    Assets, EventReader, Font, Local, Mesh, Query, Res, StandardMaterial, With,
};
use std::time::Instant;

pub(crate) fn count_assets(
    font_assets: Option<Res<Assets<Font>>>,
    map_assets: Option<Res<Assets<MapAsset>>>,
    map_memory_assets: Option<Res<Assets<MapMemoryAsset>>>,
    overmap_assets: Option<Res<Assets<OvermapAsset>>>,
    overmap_buffer_assets: Option<Res<Assets<OvermapBufferAsset>>>,
    materials: Option<Res<Assets<StandardMaterial>>>,
    meshes: Option<Res<Assets<Mesh>>>,
    mut last_counts: Local<Vec<usize>>,
) {
    if !cfg!(debug_assertions) {
        return;
    }

    let start = Instant::now();

    let counts = vec![
        font_assets.map_or(0, |a| a.len()),
        map_assets.map_or(0, |a| a.len()),
        map_memory_assets.map_or(0, |a| a.len()),
        overmap_assets.map_or(0, |a| a.len()),
        overmap_buffer_assets.map_or(0, |a| a.len()),
        materials.map_or(0, |a| a.len()),
        meshes.map_or(0, |a| a.len()),
    ];

    if *last_counts != counts && counts.iter().any(|c| 0 < *c) {
        println!("{} font assets", counts[0]);
        println!("{} map assets", counts[1]);
        println!("{} map memory assets", counts[2]);
        println!("{} overmap assets", counts[3]);
        println!("{} overmap buffer assets", counts[4]);
        println!("{} material assets", counts[5]);
        println!("{} mesh assets", counts[6]);

        *last_counts = counts;
    }

    log_if_slow("count_assets", start);
}

#[expect(clippy::needless_pass_by_value)]
pub(crate) fn count_pos(
    zone_levels: Query<(), With<ZoneLevel>>,
    subzone_levels: Query<(), With<SubzoneLevel>>,
    pos: Query<(), With<Pos>>,
    mut last_counts: Local<[usize; 3]>,
) {
    if !cfg!(debug_assertions) {
        return;
    }

    let start = Instant::now();

    let zone_levels = zone_levels.iter().len();
    let subzone_levels = subzone_levels.iter().len();
    let pos = pos.iter().len();

    let counts = [zone_levels, subzone_levels, pos];

    if *last_counts != counts && counts.iter().any(|c| 0 < *c) {
        println!("{subzone_levels} zone levels, {zone_levels} subzone levels, and {pos} positions");

        *last_counts = counts;
    }

    log_if_slow("count_pos", start);
}

pub(crate) fn check_failed_asset_loading(
    mut fails: EventReader<UntypedAssetLoadFailedEvent>,
) {
    let start = Instant::now();

    for fail in fails.read() {
        eprintln!("Failed to load asset {}: {:#?}", fail.path, &fail.error);
    }

    log_if_slow("check_failed_asset_loading", start);
}

#[cfg(feature = "log_archetypes")]
#[expect(clippy::needless_pass_by_value)]
pub(crate) fn log_archetypes(world: &mut World) {
    let component_names = world
        .components()
        .iter()
        .map(|component| {
            (component.id(), {
                let name = component.name();
                let (base, brackets) = name.split_once('<').unwrap_or((name, ""));
                let short_base = base.rsplit_once(':').unwrap_or(("", base)).1;
                String::from(short_base) + (if brackets.is_empty() { "" } else { "<" }) + brackets
            })
        })
        .collect::<HashMap<_, _>>();

    for archetype in world.archetypes().iter() {
        if !archetype.is_empty() {
            println!(
                "{:?} {:?} {:?}",
                archetype.id(),
                archetype.len(),
                archetype
                    .components()
                    .map(|component| component_names
                        .get(&component)
                        .cloned()
                        .unwrap_or(String::from("[???]")))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
}
