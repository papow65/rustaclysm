use crate::prelude::*;
use bevy::{ecs::system::Resource, utils::HashMap};
use glob::glob;
use serde::de::DeserializeOwned;
use serde_json::map::Entry;
use std::{fs::read_to_string, path::PathBuf};

#[derive(Resource)]
pub(crate) struct Infos {
    characters: HashMap<ObjectId, CharacterInfo>,
    fields: HashMap<ObjectId, FieldInfo>,
    furniture: HashMap<ObjectId, FurnitureInfo>,
    items: HashMap<ObjectId, ItemInfo>,
    terrain: HashMap<ObjectId, TerrainInfo>,
    zone_levels: HashMap<ObjectId, OvermapInfo>,
    migrations: HashMap<ObjectId, Migration>,
    item_groups: HashMap<ObjectId, ItemGroup>,
}

impl Infos {
    fn literals() -> HashMap<TypeId, HashMap<String, serde_json::Map<String, serde_json::Value>>> {
        let mut literals = HashMap::default();
        for type_ids in TypeId::all() {
            for type_id in type_ids.iter() {
                literals.insert(type_id.clone(), HashMap::default());
            }
        }

        let json_path = Paths::data_path().join("json");
        let patterns = [
            json_path.join("field_type.json"),
            json_path
                .join("furniture_and_terrain")
                .join("**")
                .join("*.json"),
            json_path.join("items").join("**").join("*.json"),
            json_path.join("monsters").join("**").join("*.json"),
            json_path.join("vehicleparts").join("**").join("*.json"),
        ];
        let json_paths = patterns
            .iter()
            .map(|pattern| pattern.as_path().to_str().expect("ASCII path"))
            .flat_map(|pattern| {
                println!("Searching {pattern} for info files");
                glob(pattern).expect("Failed to read glob pattern")
            });
        let mut file_count = 0;
        let mut info_count = 0;
        for json_path in json_paths {
            let json_path = json_path.expect("problem with json path for infos");
            if json_path.ends_with("default_blacklist.json")
                || json_path.ends_with("dreams.json")
                || json_path.ends_with("effect_on_condition.json")
            {
                continue;
            }
            let file_contents = read_to_string(&json_path)
                .unwrap_or_else(|_| panic!("Could not read {}", json_path.display()));
            let contents = serde_json::from_str::<Vec<serde_json::Map<String, serde_json::Value>>>(
                file_contents.as_str(),
            );
            let contents = contents.expect("Failed loading infos");
            file_count += 1;

            for content in contents {
                let type_ = content.get("type").expect("type present");
                let type_ = TypeId::get(type_.as_str().expect("string value for type"));
                if type_ == TypeId::get("mapgen") || content.get("from_variant").is_some() {
                    continue; // TODO
                }

                let mut ids = Vec::new();
                match id_value(&content, &json_path) {
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
                //println!("Info abount {:?} > {:?}", &type_, &ids);
                let by_type = literals.get_mut(&type_.clone()).unwrap();
                for id in ids {
                    assert!(by_type.get(&id).is_none(), "double entry for {:?}", &id);
                    by_type.insert(id.clone(), content.clone());
                    info_count += 1;
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
                //println!("Info abount {:?} > aliases {:?}", &type_, &aliases);
                for alias in aliases {
                    // Duplicates possible
                    if by_type.get(&alias).is_none() {
                        by_type.insert(alias.clone(), content.clone());
                        info_count += 1;
                    }
                }
            }
        }
        println!("Found {info_count} ids in {file_count} info files");
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
                //println!("{:?}", &object_id);
                let mut enriched = literal.clone();
                let mut ancestors = vec![object_id.clone()];
                while let Some(copy_from) = enriched.remove("copy-from") {
                    //println!("Copy from {:?}", &copy_from);
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
                enriched.remove("from");
                enriched.remove("abstract");
                enriched.remove("copy-from");

                if TypeId::MIGRATION.contains(type_id) {
                    // new_id -> replace <- to
                    if let Some(new_id) = enriched.remove("new_id") {
                        assert!(!enriched.contains_key("to"));
                        let replace = enriched.entry("replace");
                        assert!(matches!(replace, Entry::Vacant { .. }));
                        replace.or_insert(new_id);
                    } else if let Some(to) = enriched.remove("to") {
                        assert!(!enriched.contains_key("new_id"));
                        let replace = enriched.entry("replace");
                        assert!(matches!(replace, Entry::Vacant { .. }));
                        replace.or_insert(to);
                    }
                }

                enriched_of_type.insert(ObjectId::new(object_id), enriched);
            }
        }
        enricheds
    }

    pub(crate) fn new() -> Self {
        let mut enricheds = Self::enricheds();
        let mut this = Self {
            characters: Self::extract(&mut enricheds, TypeId::CHARACTER),
            fields: Self::extract(&mut enricheds, TypeId::FIELD),
            furniture: Self::extract(&mut enricheds, TypeId::FURNITURE),
            items: Self::extract(&mut enricheds, TypeId::ITEM),
            terrain: Self::extract(&mut enricheds, TypeId::TERRAIN),
            zone_levels: Self::extract(&mut enricheds, TypeId::OVERMAP),
            migrations: Self::extract(&mut enricheds, TypeId::MIGRATION),
            item_groups: Self::extract(&mut enricheds, TypeId::ITEM_GROUP),
        };

        this.characters.insert(
            ObjectId::new("human"),
            CharacterInfo {
                name: ItemName::from(CddaItemName::Simple(String::from("Human"))),
                default_faction: String::from("human"),
                looks_like: Some(ObjectId::new("overlay_male_mutation_SKIN_TAN")),
                volume: Some(Volume::from(String::from("80 l"))),
                mass: Some(Mass::from(String::from("80 kg"))),
                hp: Some(100),
                speed: 100,
                melee_dice: 2,
                melee_dice_sides: 4,
                flags: Flags::default(),
                extra: HashMap::default(),
            },
        );

        this
    }

    pub(crate) fn character<'a>(&'a self, id: &'a ObjectId) -> Option<&'a CharacterInfo> {
        let id = self.migrations.get(id).map_or(id, |m| &m.replace);
        self.characters.get(id)
    }

    pub(crate) fn field<'a>(&'a self, id: &'a ObjectId) -> Option<&'a FieldInfo> {
        let id = self.migrations.get(id).map_or(id, |m| &m.replace);
        self.fields.get(id)
    }

    pub(crate) fn furniture<'a>(&'a self, id: &'a ObjectId) -> Option<&'a FurnitureInfo> {
        let id = self.migrations.get(id).map_or(id, |m| &m.replace);
        self.furniture.get(id)
    }

    pub(crate) fn item<'a>(&'a self, id: &'a ObjectId) -> Option<&'a ItemInfo> {
        let id = self.migrations.get(id).map_or(id, |m| &m.replace);
        self.items.get(id)
    }

    pub(crate) fn item_group<'a>(&'a self, id: &'a ObjectId) -> Option<&'a ItemGroup> {
        let id = self.migrations.get(id).map_or(id, |m| &m.replace);
        self.item_groups.get(id)
    }

    pub(crate) fn terrain<'a>(&'a self, id: &'a ObjectId) -> Option<&'a TerrainInfo> {
        let id = self.migrations.get(id).map_or(id, |m| &m.replace);
        self.terrain.get(id)
    }

    fn looks_like(&self, definition: &ObjectDefinition) -> Option<&ObjectId> {
        match definition.category {
            ObjectCategory::Character => self
                .characters
                .get(&definition.id)
                .and_then(|o| o.looks_like.as_ref()),
            ObjectCategory::Item => self
                .items
                .get(&definition.id)
                .and_then(|o| o.looks_like.as_ref()),
            ObjectCategory::Field => self
                .fields
                .get(&definition.id)
                .and_then(|o| o.looks_like.as_ref()),
            ObjectCategory::Furniture => self
                .furniture
                .get(&definition.id)
                .and_then(|o| o.looks_like.as_ref()),
            ObjectCategory::Terrain => self
                .terrain
                .get(&definition.id)
                .and_then(|o| o.looks_like.as_ref()),
            ObjectCategory::ZoneLevel => self
                .zone_levels
                .get(&definition.id)
                .and_then(|o| o.looks_like.as_ref()),
            _ => unimplemented!("{:?}", definition),
        }
    }

    pub(crate) fn variants(&self, definition: &ObjectDefinition) -> Vec<ObjectId> {
        if definition.category == ObjectCategory::Meta {
            return vec![definition.id.clone()];
        }

        let current_id;
        let mut current_definition;
        let mut current_definition_ref = definition;
        if definition.category == ObjectCategory::ZoneLevel {
            current_id = current_definition_ref.id.truncate();
            current_definition = ObjectDefinition {
                category: definition.category.clone(),
                id: current_id,
            };
            current_definition_ref = &current_definition;
        }

        let mut variants = vec![
            current_definition_ref.id.suffix("_season_summer"),
            current_definition_ref.id.clone(),
        ];

        while let Some(other) = self.looks_like(current_definition_ref) {
            if variants.contains(other) {
                //eprintln!("Variants {:?} already contains {:?}", &variants, &other); // TODO
                break;
            }
            variants.push(other.suffix("_season_summer"));
            variants.push(other.clone());
            current_definition = ObjectDefinition {
                category: definition.category.clone(),
                id: other.clone(),
            };
            current_definition_ref = &current_definition;
        }
        variants
    }

    #[must_use]
    pub(crate) fn name(&self, definition: &ObjectDefinition) -> ItemName {
        let name = match definition.category {
            ObjectCategory::Character => self.characters.get(&definition.id).map(|o| &o.name),
            ObjectCategory::Item => self.items.get(&definition.id).map(|o| &o.name),
            ObjectCategory::Field => self.fields.get(&definition.id).map(FieldInfo::name),
            ObjectCategory::Furniture => self.furniture.get(&definition.id).map(|o| &o.name),
            ObjectCategory::Terrain => self.terrain.get(&definition.id).map(|o| &o.name),
            ObjectCategory::ZoneLevel => self.zone_levels.get(&definition.id).map(|o| &o.name),
            _ => unimplemented!("{:?}", definition.category),
        };

        name.cloned().unwrap_or(ItemName::from(CddaItemName::Simple(
            definition.id.fallback_name(),
        )))
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

fn id_value<'a>(
    content: &'a serde_json::Map<String, serde_json::Value>,
    json_path: &'a PathBuf,
) -> &'a serde_json::Value {
    let mut count = 0;
    if content.get("id").is_some() {
        count += 1;
    }
    if content.get("abstract").is_some() {
        count += 1;
    }
    if content.get("from").is_some() {
        count += 1;
    }
    assert_eq!(count, 1, "{json_path:?}");
    content
        .get("id")
        .or_else(|| content.get("abstract"))
        .or_else(|| content.get("from"))
        .expect("id, abstract, or from")
}
