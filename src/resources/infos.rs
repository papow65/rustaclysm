use crate::prelude::*;
use bevy::{ecs::system::Resource, utils::HashMap};
use glob::glob;
use serde::de::DeserializeOwned;
use std::fs::read_to_string;

#[derive(Resource)]
pub(crate) struct Infos {
    items: HashMap<ObjectId, CddaItemInfo>,
    furniture: HashMap<ObjectId, CddaFurnitureInfo>,
    terrain: HashMap<ObjectId, CddaTerrainInfo>,
    zone_level: HashMap<ObjectId, CddaOvermapInfo>,
}

impl Infos {
    fn literals() -> HashMap<TypeId, HashMap<String, serde_json::Map<String, serde_json::Value>>> {
        let mut literals = HashMap::default();
        for type_ids in TypeId::all() {
            for type_id in type_ids.iter() {
                literals.insert(type_id.clone(), HashMap::default());
            }
        }

        let pattern = Paths::data_path().join("json").join("**").join("*.json");
        let pattern = pattern.as_path().to_str().expect("ASCII path");
        for json_path in glob(pattern).expect("Failed to read glob pattern") {
            let json_path = json_path.expect("problem with json path for infos");
            if !(json_path.display().to_string().contains("/items/")
                || json_path
                    .display()
                    .to_string()
                    .contains("/furniture_and_terrain/")
                || json_path.display().to_string().contains("/vehicleparts/")
                || json_path.ends_with("field_type.json"))
                || json_path.ends_with("default_blacklist.json")
                || json_path.ends_with("dreams.json")
                || json_path.ends_with("migration.json")
                || json_path.ends_with("effect_on_condition.json")
            {
                continue;
            }
            let file_contents = read_to_string(&json_path)
                .unwrap_or_else(|_| panic!("Could not read {}", json_path.display()));
            println!("Found info file: {}", json_path.display());
            let contents = serde_json::from_str::<Vec<serde_json::Map<String, serde_json::Value>>>(
                file_contents.as_str(),
            );
            let contents = contents.expect("Failed loading infos");

            for content in contents {
                let type_ = content.get("type").expect("type present");
                let type_ = TypeId::get(type_.as_str().expect("string value for type"));

                assert_ne!(
                    content.get("id").is_some(),
                    content.get("abstract").is_some()
                );

                let mut ids = Vec::new();
                match content
                    .get("id")
                    .unwrap_or_else(|| content.get("abstract").expect("id or abstract"))
                {
                    serde_json::Value::String(id) => {
                        ids.push(String::from(id.as_str()));
                    }
                    serde_json::Value::Array(a) => {
                        for id in a {
                            ids.push(String::from(id.as_str().unwrap()));
                        }
                    }
                    _ => {
                        unreachable!();
                    }
                }
                println!("Info abount {:?} > {:?}", &type_, &ids);
                let by_type = literals.get_mut(&type_.clone()).unwrap();
                for id in ids {
                    assert!(by_type.get(&id).is_none(), "double entry for {:?}", &id);
                    by_type.insert(id.clone(), content.clone());
                }

                let mut aliases = Vec::new();
                if let Some(alias) = content.get("alias") {
                    match alias {
                        serde_json::Value::String(id) => {
                            aliases.push(String::from(id.as_str()));
                        }
                        serde_json::Value::Array(a) => {
                            for id in a {
                                aliases.push(String::from(id.as_str().unwrap()));
                            }
                        }
                        _ => {
                            unreachable!();
                        }
                    }
                }
                println!("Info abount {:?} > aliases {:?}", &type_, &aliases);
                for alias in aliases {
                    // Duplicates possible
                    if by_type.get(&alias).is_none() {
                        by_type.insert(alias.clone(), content.clone());
                    }
                }
            }
        }
        assert!(!literals.is_empty());
        literals
    }

    fn enricheds() -> HashMap<TypeId, HashMap<ObjectId, serde_json::Map<String, serde_json::Value>>>
    {
        let mut enricheds = HashMap::default();
        let literals = Self::literals();
        for (type_id, literal_entry) in &literals {
            let enriched_of_type = enricheds
                .entry(type_id.clone())
                .or_insert_with(HashMap::default);
            for (object_id, literal) in literal_entry {
                println!("{:?}", &object_id);
                let mut enriched = literal.clone();
                let mut ancestors = vec![object_id.clone()];
                while let Some(copy_from) = enriched.remove("copy-from") {
                    println!("Copy from {:?}", &copy_from);
                    let copy_from = copy_from.as_str().expect("string value for copy-from");
                    ancestors.push(String::from(copy_from));
                    assert!(ancestors.len() < 10, "{ancestors:?}");
                    let literals = &literals;
                    let copy_from = literal_entry.get(copy_from).unwrap_or_else(|| {
                        let mut other_types = literals
                            .into_iter()
                            .filter_map(|(_, literal_entry)| literal_entry.get(copy_from));
                        let single = other_types.next().unwrap_or_else(|| {
                            panic!(
                                "copy-from {copy_from:?} not found for ({:?}) {:?}",
                                &type_id, &copy_from
                            )
                        });
                        assert!(
                            other_types.next().is_none(),
                            "Multiple copy-from {copy_from:?} found for {:?} {:?}",
                            &type_id,
                            &object_id
                        );
                        single
                    });
                    for (key, value) in copy_from {
                        enriched.entry(key.clone()).or_insert(value.clone());
                    }
                }
                enriched.remove("id");
                enriched.remove("abstract");
                enriched.remove("copy-from");

                enriched_of_type.insert(ObjectId::new(object_id), enriched);
            }
        }
        enricheds
    }

    pub(crate) fn new() -> Self {
        let mut enricheds = Self::enricheds();
        Self {
            items: Self::extract(&mut enricheds, TypeId::ITEM),
            furniture: Self::extract(&mut enricheds, TypeId::FURNITURE),
            terrain: Self::extract(&mut enricheds, TypeId::TERRAIN),
            zone_level: Self::extract(&mut enricheds, TypeId::OVERMAP),
        }
    }

    pub(crate) fn item<'a>(&'a self, id: &'a ObjectId) -> Option<&'a CddaItemInfo> {
        self.items.get(id)
    }

    pub(crate) fn furniture<'a>(&'a self, id: &'a ObjectId) -> Option<&'a CddaFurnitureInfo> {
        self.furniture.get(id)
    }

    pub(crate) fn terrain<'a>(&'a self, id: &'a ObjectId) -> Option<&'a CddaTerrainInfo> {
        self.terrain.get(id)
    }

    fn looks_like(&self, definition: &ObjectDefinition) -> Option<&ObjectId> {
        match definition.specifier {
            ObjectSpecifier::Item => self
                .items
                .get(definition.id)
                .and_then(|o| o.looks_like.as_ref()),
            ObjectSpecifier::Furniture => self
                .furniture
                .get(definition.id)
                .and_then(|o| o.looks_like.as_ref()),
            ObjectSpecifier::Terrain => self
                .terrain
                .get(definition.id)
                .and_then(CddaTerrainInfo::looks_like),
            ObjectSpecifier::ZoneLevel => self
                .zone_level
                .get(definition.id)
                .and_then(|o| o.looks_like.as_ref()),
            _ => unimplemented!("{:?}", definition.specifier),
        }
    }

    pub(crate) fn variants(&self, definition: &ObjectDefinition) -> Vec<ObjectId> {
        let current_id;
        let mut current_definition;
        let mut current_definition_ref = definition;
        if definition.specifier == ObjectSpecifier::ZoneLevel {
            current_id = current_definition_ref.id.truncate();
            current_definition = ObjectDefinition {
                id: &current_id,
                specifier: definition.specifier.clone(),
            };
            current_definition_ref = &current_definition;
        }

        let mut variants = vec![
            current_definition_ref.id.suffix("_season_summer"),
            current_definition_ref.id.clone(),
        ];

        while let Some(other) = self.looks_like(current_definition_ref) {
            variants.push(other.suffix("_season_summer"));
            variants.push(other.clone());
            current_definition = ObjectDefinition {
                id: other,
                specifier: definition.specifier.clone(),
            };
            current_definition_ref = &current_definition;
        }
        variants
    }

    pub(crate) fn label<'a>(&'a self, definition: &'a ObjectDefinition, amount: usize) -> Label {
        let name = match definition.specifier {
            ObjectSpecifier::Item => self.items.get(definition.id).map(|o| &o.name),
            ObjectSpecifier::Furniture => self.furniture.get(definition.id).map(|o| &o.name),
            ObjectSpecifier::Terrain => self.terrain.get(definition.id).map(CddaTerrainInfo::name),
            ObjectSpecifier::ZoneLevel => self.zone_level.get(definition.id).map(|o| &o.name),
            _ => unimplemented!("{:?}", definition.specifier),
        };

        if let Some(name) = name {
            name.to_label(amount)
        } else {
            definition.id.to_fallback_label()
        }
    }

    fn extract<T>(
        all: &mut HashMap<TypeId, HashMap<ObjectId, serde_json::Map<String, serde_json::Value>>>,
        type_ids: &[TypeId],
    ) -> HashMap<ObjectId, T>
    where
        T: DeserializeOwned,
    {
        let mut result = HashMap::default();
        for type_id in type_ids {
            let objects = all
                .remove(type_id)
                .unwrap_or_else(|| panic!("Type {type_id:?} not found"));
            for (id, object_properties) in objects {
                //println!("{:?}", &object_properties);
                let info: T = serde_json::from_value(serde_json::Value::Object(object_properties))
                    .unwrap_or_else(|e| panic!("{:?} {:?}", &id, &e));
                result.insert(id, info);
            }
        }

        result
    }
}
