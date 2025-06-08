use crate::{Error, TypeId};
use bevy::platform::collections::HashMap;
use bevy::prelude::{debug, error, warn};
use bevy::tasks::IoTaskPool;
use cdda_json_files::UntypedInfoId;
use fastrand::alphabetic;
use glob::glob;
use serde::Deserialize;
use std::fs::{File, read_to_string};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::{env, io::Write as _};
use util::AssetPaths;

#[derive(Debug, Deserialize)]
struct Typed {
    #[serde(rename = "type")]
    type_id: TypeId,

    #[serde(flatten)]
    fields: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug)]
enum Proto {
    Primary {
        fields: Arc<serde_json::Map<String, serde_json::Value>>,
        alias_ids: Vec<UntypedInfoId>,
    },
    Alias {
        fields: Arc<serde_json::Map<String, serde_json::Value>>,
    },
}

impl Proto {
    const fn fields(&self) -> &Arc<serde_json::Map<String, serde_json::Value>> {
        match self {
            Self::Primary { fields, .. } | Self::Alias { fields } => fields,
        }
    }
}

#[derive(Debug)]
pub(super) struct Enriched {
    pub(super) fields: serde_json::Map<String, serde_json::Value>,
    pub(super) alias_ids: Vec<UntypedInfoId>,
}

#[derive(Default)]
pub(super) struct ParsedJson {
    /// [`TypeId`] -> [`UntypedInfoId`] -> prototype info
    objects_by_type: Mutex<HashMap<TypeId, HashMap<UntypedInfoId, Proto>>>,
}

impl ParsedJson {
    fn json_infos_paths() -> impl Iterator<Item = PathBuf> {
        let json_file_pattern = AssetPaths::data().join("json").join("**").join("*.json");
        let json_file_pattern = json_file_pattern
            .as_path()
            .to_str()
            .expect("Path pattern should be valid UTF-8");
        debug!("Searching {json_file_pattern} for info files");
        glob(json_file_pattern)
            .expect("Glob pattern should match some readable paths")
            .map(|json_path_result| json_path_result.expect("JSON path should be valid"))
    }

    fn load() -> HashMap<TypeId, HashMap<UntypedInfoId, Proto>> {
        let this = Self::default();
        let parsed_file_count = AtomicUsize::new(0);
        let skipped_file_count = AtomicUsize::new(0);
        let skipped_id_count = AtomicUsize::new(0);

        IoTaskPool::get().scope(|s| {
            let this = &this;
            let parsed_file_count = &parsed_file_count;
            let skipped_file_count = &skipped_file_count;
            let skipped_id_count = &skipped_id_count;
            for json_path in Self::json_infos_paths() {
                s.spawn(async move {
                    match this.parse_json_info_file(&json_path, skipped_id_count) {
                        Ok(()) => {
                            parsed_file_count.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(error) => {
                            error!("Error while processing {json_path:?}: {error:#?}");
                            skipped_file_count.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                });
            }
        });

        let objects_by_type = this
            .objects_by_type
            .into_inner()
            .expect("Mutex should be unpoisoned");

        let id_count = objects_by_type.values().map(HashMap::len).sum::<usize>();
        debug!(
            "Found {id_count} ids ({skipped_id_count:?} skipped ids) in {parsed_file_count:?} info files ({skipped_file_count:?} skipped files)"
        );
        assert!(
            !objects_by_type.is_empty(),
            "Some info should have been found"
        );
        objects_by_type
    }

    fn parse_json_info_file(
        &self,
        json_path: &Path,
        skipped_count: &AtomicUsize,
    ) -> Result<(), Error> {
        //trace!("Parsing {json_path:?}...");
        let file_contents = read_to_string(json_path)?;

        let contents = match serde_json::from_str::<Vec<Typed>>(file_contents.as_str()) {
            Ok(contents) => contents,
            Err(error) => {
                // Maybe is one of the few of non-list files?
                let Ok(content) = serde_json::from_str::<Typed>(file_contents.as_str()) else {
                    // The first match attempt was the most likely to succeed, so its error is most relevant.
                    return Err(error.into());
                };

                vec![content]
            }
        };

        for content in contents {
            if content
                .fields
                .get("obsolete")
                .is_some_and(|value| value.as_bool().unwrap_or(false))
            {
                //trace!("Skipping obsolete info in {json_path:?}");
                skipped_count.fetch_add(1, Ordering::Relaxed);
                continue;
            }
            if !content.type_id.in_use() {
                skipped_count.fetch_add(1, Ordering::Relaxed);
                continue; // TODO
            }

            //trace!("Info abount {:?} > {:?}", &type_, &ids);
            let mut hash_map = self
                .objects_by_type
                .lock()
                .expect("Mutex should be unpoisoned");
            let by_type = hash_map.entry(content.type_id).or_default();
            load_ids(content.fields, by_type, content.type_id, json_path);
        }

        Ok(())
    }

    /// [`TypeId`] -> [`UntypedInfoId`] -> property name -> property value
    pub(super) fn enriched() -> HashMap<TypeId, HashMap<UntypedInfoId, Enriched>> {
        let mut enriched_json_infos = HashMap::default();
        let objects_by_type = &Self::load();
        for (&type_id, literal_entry) in objects_by_type {
            let enriched_of_type = enriched_json_infos
                .entry(type_id)
                .or_insert_with(HashMap::default);
            'enricheds: for (object_id, literal) in literal_entry {
                let Proto::Primary { fields, alias_ids } = literal else {
                    continue;
                };
                if fields.contains_key("abstract") {
                    continue;
                }
                //trace!("{:?}", &object_id);
                let mut enriched = (**fields).clone();
                let mut ancestors = vec![object_id.clone()];
                while let Some(copy_from) = enriched.remove("copy-from") {
                    //trace!("Copy from {:?}", &copy_from);
                    let copy_from = UntypedInfoId::new(
                        copy_from
                            .as_str()
                            .expect("'copy-from' should have a string value"),
                    );
                    ancestors.push(copy_from.clone());
                    assert!(ancestors.len() < 10, "{ancestors:?}");
                    let copy_from = if let Some(found) = literal_entry.get(&copy_from) {
                        found
                    } else {
                        let mut other_types = objects_by_type
                            .into_iter()
                            .filter(|(type_, _)| **type_ != TypeId::Recipe)
                            .filter_map(|(_, literal_entry)| literal_entry.get(&copy_from));
                        let Some(single) = other_types.next() else {
                            error!(
                                "copy-from {copy_from:?} not found for ({:?}) {:?}",
                                &type_id, &copy_from
                            );
                            continue 'enricheds;
                        };
                        if other_types.next().is_some() {
                            error!(
                                "Multiple copy-from {copy_from:?} found for {:?} {:?}",
                                &type_id, &object_id
                            );
                            continue 'enricheds;
                        }
                        single
                    };
                    for (key, value) in &**copy_from.fields() {
                        enriched.entry(key.clone()).or_insert(value.clone());
                    }
                }

                enriched.remove("abstract");
                enriched.remove("copy-from");
                enriched.remove("id_suffix");

                enriched_of_type.insert(
                    object_id.clone(),
                    Enriched {
                        fields: enriched,
                        alias_ids: alias_ids.clone(),
                    },
                );
            }
        }

        if let Ok(directory_name) = env::var("DUMP_ENRICHED") {
            for (type_id, info_map) in &enriched_json_infos {
                let json_string = serde_json::to_string_pretty(
                    &info_map
                        .values()
                        .map(|enriched| &enriched.fields)
                        .collect::<Vec<_>>(),
                )
                .expect("Serialization should succeed");
                File::create(format!("{directory_name}/enriched_{type_id:?}.json").to_lowercase())
                    .expect("File creation should succeed")
                    .write_all(json_string.as_bytes())
                    .expect("Writing should succeed");
            }
        }

        enriched_json_infos
    }
}

fn load_ids(
    mut content: serde_json::Map<String, serde_json::Value>,
    by_type: &mut HashMap<UntypedInfoId, Proto>,
    type_id: TypeId,
    json_path: &Path,
) {
    let mut ids = id_values(&content, type_id, json_path);
    if ids.is_empty() {
        error!("No ids found");
        return;
    }

    ids.append(&mut alias_values(&content, type_id, json_path));

    let mut ids = ids.into_iter().filter_map(|mut id| {
        if let Some(previous) = by_type.get(&id) {
            if content == **previous.fields() {
                //trace!("Ignoring exact duplicate info for {id:?}");
                None
            } else if type_id == TypeId::Recipe {
                //trace!("Old: {:#?}", by_type.get(&id));
                //trace!("New: {content:#?}");
                let random_string: String = [(); 16]
                .map(|()| alphabetic().to_ascii_lowercase())
                .iter()
                .collect();
                id.add_suffix(random_string.as_str());
                Some(id)
            } else {
                error!(
                    "Duplicate usage of id {id:?} in {json_path:?} detected. One will be ignored.",
                );
                None
            }
        } else {
            Some(id)
        }
    }).collect::<Vec<_>>();

    if ids.is_empty() {
        //trace!("No ids left");
        return;
    }

    let first_id = ids.swap_remove(0);
    let alias_ids = ids;

    if type_id != TypeId::VehiclePartMigration {
        content.insert(
            String::from("id"),
            serde_json::Value::String(String::from(&*first_id.fallback_name())),
        );
    }
    content.remove("alias");
    let content = Arc::new(content);

    for alias_id in &alias_ids {
        by_type.insert(
            alias_id.clone(),
            Proto::Alias {
                fields: content.clone(),
            },
        );
    }
    by_type.insert(
        first_id,
        Proto::Primary {
            fields: content,
            alias_ids,
        },
    );
}

fn id_values(
    content: &serde_json::Map<String, serde_json::Value>,
    type_id: TypeId,
    json_path: &Path,
) -> Vec<UntypedInfoId> {
    let id_suffix = content.get("id_suffix").and_then(|suffix| {
        suffix
            .as_str()
            .ok_or(())
            .inspect_err(|()| {
                error!("Unexpected if_suffix format for {type_id:?} in {json_path:?}: {suffix:?}");
            })
            .ok()
    });
    if type_id != TypeId::Recipe && id_suffix.is_some() {
        warn!("Unexpected combination of id_suffix for {type_id:?} in {json_path:?}: {content:#?}");
    }

    let from_variant =
        content.get("from_variant").and_then(|suffix| {
            suffix
        .as_str()
        .ok_or(())
        .inspect_err(|()| {
            error!("Unexpected from_variant format for {type_id:?} in {json_path:?}: {suffix:?}");
        })
        .ok()
        });
    if type_id != TypeId::ItemMigration && from_variant.is_some() {
        warn!(
            "Unexpected combination of from_variant for {type_id:?} in {json_path:?}: {content:#?}"
        );
    }

    let id = match type_id {
        TypeId::Recipe => {
            match (
                content.get("result"),
                content.get("abstract"),
                content.get("copy-from"),
            ) {
                (Some(id), None, _) | (None, Some(id), _) | (None, None, Some(id)) => id,
                _ => {
                    error!("Could not determine id for {type_id:?} in {json_path:?}: {content:#?}");
                    return Vec::new();
                }
            }
        }
        TypeId::VehiclePartMigration => {
            if let Some(from) = content.get("from") {
                from
            } else {
                error!("Could not determine id for {type_id:?} in {json_path:?}: {content:#?}");
                return Vec::new();
            }
        }
        _ => match (content.get("id"), content.get("abstract")) {
            (Some(id), None) | (None, Some(id)) => id,
            _ => {
                error!("Could not determine id for {type_id:?} in {json_path:?}: {content:#?}");
                return Vec::new();
            }
        },
    };

    match id {
        serde_json::Value::String(id) => {
            vec![UntypedInfoId::new_suffix(id, id_suffix.or(from_variant))]
        }
        serde_json::Value::Array(ids_array) if !ids_array.is_empty() => ids_array
            .iter()
            .filter_map(|id| match id {
                serde_json::Value::String(id) => Some(UntypedInfoId::new_suffix(id, id_suffix)),
                unexpected => {
                    error!(
                        "Skipping non-string id for {type_id:?} in {json_path:?}: {unexpected:?}"
                    );
                    None
                }
            })
            .collect(),
        _ => {
            error!("Unexpected id structure for {type_id:?} in {json_path:?}: {id:?}");
            Vec::new()
        }
    }
}

fn alias_values(
    content: &serde_json::Map<String, serde_json::Value>,
    type_id: TypeId,
    json_path: &Path,
) -> Vec<UntypedInfoId> {
    match content.get("alias") {
        Some(serde_json::Value::String(alias)) => {
            vec![UntypedInfoId::new(alias.as_str())]
        }
        Some(serde_json::Value::Array(aliases)) if !aliases.is_empty() => aliases
            .iter()
            .filter_map(|alias| match alias {
                serde_json::Value::String(alias) => Some(UntypedInfoId::new(alias.as_str())),
                unexpected => {
                    error!(
                        "Skipping non-string alias for {type_id:?} in {json_path:?}: {unexpected:?}"
                    );
                    None
                }
            })
            .collect(),
        Some(unexpected) => {
            error!("Skipping unexpected alias structure in {json_path:?}: {unexpected:#?}",);
            Vec::new()
        }
        None => Vec::new(),
    }
}
