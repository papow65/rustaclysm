use crate::gameplay::{TypeId, cdda::Error};
use bevy::platform_support::collections::HashMap;
use bevy::prelude::{debug, error, warn};
use cdda_json_files::UntypedInfoId;
use fastrand::alphabetic;
use glob::glob;
use serde::Deserialize;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use strum::VariantArray as _;
use util::AssetPaths;

#[derive(Debug, Deserialize)]
struct Typed {
    #[serde(rename = "type")]
    type_id: TypeId,

    #[serde(flatten)]
    fields: serde_json::Map<String, serde_json::Value>,
}

#[derive(Default)]
pub(super) struct ParsedJson {
    /// [`TypeId`] -> [`UntypedInfoId`] -> property name -> property value
    objects_by_type:
        HashMap<TypeId, HashMap<UntypedInfoId, serde_json::Map<String, serde_json::Value>>>,
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

    fn load() -> Self {
        let mut parsed_json = Self::default();
        for type_id in TypeId::VARIANTS {
            parsed_json
                .objects_by_type
                .insert(*type_id, HashMap::default());
        }

        let mut parsed_file_count = 0;
        let mut skipped_count = 0;
        for json_path in Self::json_infos_paths() {
            match parsed_json.parse_json_info_file(&json_path, &mut skipped_count) {
                Ok(()) => {
                    parsed_file_count += 1;
                }
                Err(error) => {
                    error!("Error while processing {json_path:?}: {error:#?}");
                }
            }
        }

        let id_count = parsed_json
            .objects_by_type
            .values()
            .map(HashMap::len)
            .sum::<usize>();
        debug!("Found {id_count} ids ({skipped_count} skipped) in {parsed_file_count} info files");
        assert!(
            !parsed_json.objects_by_type.is_empty(),
            "Some info should have been found"
        );
        parsed_json
    }

    fn parse_json_info_file(
        &mut self,
        json_path: &Path,
        skipped_count: &mut usize,
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
                *skipped_count += 1;
                continue;
            }
            if !content.type_id.in_use() || content.fields.get("from_variant").is_some() {
                *skipped_count += 1;
                continue; // TODO
            }

            //trace!("Info abount {:?} > {:?}", &type_, &ids);
            let by_type = self
                .objects_by_type
                .get_mut(&content.type_id)
                .expect("All TypeId variants should be present");

            load_ids(&content.fields, by_type, content.type_id, json_path);
            load_aliases(&content.fields, by_type, json_path);
        }

        Ok(())
    }

    /// [`TypeId`] -> [`UntypedInfoId`] -> property name -> property value
    pub(super) fn enriched()
    -> HashMap<TypeId, HashMap<UntypedInfoId, serde_json::Map<String, serde_json::Value>>> {
        let mut enriched_json_infos = HashMap::default();
        let objects_by_type = &Self::load().objects_by_type;
        for (&type_id, literal_entry) in objects_by_type {
            let enriched_of_type = enriched_json_infos
                .entry(type_id)
                .or_insert_with(HashMap::default);
            'enricheds: for (object_id, literal) in literal_entry {
                if literal.contains_key("abstract") {
                    continue;
                }
                //trace!("{:?}", &object_id);
                let mut enriched = literal.clone();
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
                    for (key, value) in copy_from {
                        enriched.entry(key.clone()).or_insert(value.clone());
                    }
                }

                enriched.remove("copy-from");
                if type_id == TypeId::Recipe {
                    set_recipe_id(&mut enriched);
                }

                enriched_of_type.insert(object_id.clone(), enriched);
            }
        }
        enriched_json_infos
    }
}

fn load_ids(
    content: &serde_json::Map<String, serde_json::Value>,
    by_type: &mut HashMap<UntypedInfoId, serde_json::Map<String, serde_json::Value>>,
    type_id: TypeId,
    json_path: &Path,
) {
    let ids = id_values(content, type_id, json_path);
    let ids_len = ids.len();
    for mut id in ids {
        if let Some(previous) = by_type.get(&id) {
            if content == previous {
                //trace!("Ignoring exact duplicate info for {id:?}");
                continue;
            } else if type_id == TypeId::Recipe {
                //trace!("Old: {:#?}", by_type.get(&id));
                //trace!("New: {content:#?}");
                let random_string: String = [(); 16]
                    .map(|()| alphabetic().to_ascii_lowercase())
                    .iter()
                    .collect();
                id.add_suffix(random_string.as_str());
            } else {
                error!(
                    "Duplicate usage of id {id:?} in {json_path:?} detected. One will be ignored.",
                );
                continue;
            }
        }

        let mut content = content.clone();
        if 1 < ids_len && type_id != TypeId::Recipe {
            content.insert(
                String::from("id"),
                serde_json::Value::String(String::from(&*id.fallback_name())),
            );
        }
        by_type.insert(id.clone(), content);
    }
}

fn load_aliases(
    content: &serde_json::Map<String, serde_json::Value>,
    by_type: &mut HashMap<UntypedInfoId, serde_json::Map<String, serde_json::Value>>,
    json_path: &Path,
) {
    let mut aliases = Vec::new();
    if let Some(alias) = content.get("alias") {
        match alias {
            serde_json::Value::String(id) => {
                aliases.push(UntypedInfoId::new(id.as_str()));
            }
            serde_json::Value::Array(a) => {
                for id in a {
                    if let Some(id) = id.as_str() {
                        aliases.push(UntypedInfoId::new(id));
                    } else {
                        error!("Skipping unexpected alias in {json_path:?}: {alias:#?}");
                    }
                }
            }
            _ => {
                error!("Skipping unexpected alias structure in {json_path:?}: {alias:#?}",);
            }
        }
    }
    //trace!("Info abount {:?} > aliases {:?}", &type_, &aliases);
    for alias in aliases {
        // Duplicates possible
        if by_type.get(&alias).is_none() {
            by_type.insert(alias.clone(), content.clone());
        }
    }
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

    let id = if type_id == TypeId::Recipe {
        match (
            content.get("result"),
            content.get("abstract"),
            content.get("copy-from"),
        ) {
            (Some(id), None, _) | (None, Some(id), _) | (None, None, Some(id)) => id,
            _ => {
                error!("Could not determine id for recipe in {json_path:?}: {content:#?}");
                return Vec::new();
            }
        }
    } else {
        if id_suffix.is_some() {
            warn!(
                "Unexpected combination of id_suffix for {type_id:?} in {json_path:?}: {content:#?}"
            );
        }

        match (
            content.get("id"),
            content.get("abstract"),
            content.get("from"),
        ) {
            (Some(id), None, None) | (None, Some(id), None) | (None, None, Some(id)) => id,
            _ => {
                error!("Could not determine id for {type_id:?} in {json_path:?}: {content:#?}");
                return Vec::new();
            }
        }
    };

    match id {
        serde_json::Value::String(id) => {
            vec![UntypedInfoId::new_suffix(id, id_suffix)]
        }
        serde_json::Value::Array(ids_array) if !ids_array.is_empty() => ids_array
            .iter()
            .filter_map(|id| match id {
                serde_json::Value::String(id) => Some(UntypedInfoId::new_suffix(id, id_suffix)),
                id => {
                    error!("Skipping non-string id for {type_id:?} in {json_path:?}: {id:?}");
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

fn set_recipe_id(enriched: &mut serde_json::Map<String, serde_json::Value>) {
    if let Some(recipe_id) = enriched.get("id") {
        warn!("Recipe should not have an id: {recipe_id:?}");
    } else if let Some(result) = enriched.get("result").cloned() {
        if let Some(result_str) = result.as_str() {
            let id = UntypedInfoId::new_suffix(
                result_str,
                enriched.get("id_suffix").and_then(|s| s.as_str()),
            )
            .fallback_name();
            let id = serde_json::Value::String(String::from(&*id));
            enriched.entry("id").or_insert(id);
        } else {
            error!("Recipe result should be a string: {result:#?}");
        }
    } else {
        error!("Recipe should have a result: {enriched:#?}");
    }
}
