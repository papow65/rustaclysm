use crate::gameplay::{ObjectCategory, ObjectDefinition, TypeId};
use crate::util::{AssetPaths, AsyncNew};
use bevy::{ecs::system::Resource, utils::HashMap};
use cdda_json_files::{
    Alternative, Ammo, BionicItem, Book, CddaItemName, CharacterInfo, Clothing, Comestible,
    CommonItemInfo, Engine, FieldInfo, Flags, FurnitureInfo, GenericItem, Gun, Gunmod, ItemGroup,
    ItemName, Magazine, Migration, ObjectId, OvermapInfo, PetArmor, Quality, Recipe, Requirement,
    TerrainInfo, Tool, ToolClothing, Toolmod, Using, UsingKind, VehiclePartInfo, Wheel,
};
use glob::glob;
use serde::de::DeserializeOwned;
use serde_json::map::Entry;
use std::sync::Arc;
use std::{any::type_name, fs::read_to_string, path::PathBuf, time::Instant};
use units::{Mass, Volume};

#[derive(Resource)]
pub(crate) struct Infos {
    ammos: HashMap<ObjectId, Ammo>,
    bionic_items: HashMap<ObjectId, BionicItem>,
    books: HashMap<ObjectId, Book>,
    characters: HashMap<ObjectId, CharacterInfo>,
    clothings: HashMap<ObjectId, Clothing>,
    comestibles: HashMap<ObjectId, Comestible>,
    common_item_info: HashMap<ObjectId, CommonItemInfo>,
    engines: HashMap<ObjectId, Engine>,
    fields: HashMap<ObjectId, FieldInfo>,
    furniture: HashMap<ObjectId, FurnitureInfo>,
    genenric_items: HashMap<ObjectId, GenericItem>,
    guns: HashMap<ObjectId, Gun>,
    gunmods: HashMap<ObjectId, Gunmod>,
    item_groups: HashMap<ObjectId, ItemGroup>,
    magazines: HashMap<ObjectId, Magazine>,
    migrations: HashMap<ObjectId, Migration>,
    pet_armors: HashMap<ObjectId, PetArmor>,
    qualities: HashMap<ObjectId, Quality>,
    recipes: HashMap<ObjectId, Recipe>,
    requirements: HashMap<ObjectId, Requirement>,
    terrain: HashMap<ObjectId, TerrainInfo>,
    tools: HashMap<ObjectId, Tool>,
    tool_clothings: HashMap<ObjectId, ToolClothing>,
    toolmods: HashMap<ObjectId, Toolmod>,
    vehicle_parts: HashMap<ObjectId, VehiclePartInfo>,
    wheels: HashMap<ObjectId, Wheel>,
    zone_levels: HashMap<ObjectId, OvermapInfo>,
}

impl Infos {
    fn literals_paths() -> Vec<PathBuf> {
        let json_path = AssetPaths::data().join("json");
        let patterns = [
            json_path.join("field_type.json"),
            json_path.join("tool_qualities.json"),
            json_path
                .join("furniture_and_terrain")
                .join("**")
                .join("*.json"),
            json_path.join("items").join("**").join("*.json"),
            json_path.join("monsters").join("**").join("*.json"),
            json_path.join("obsoletion").join("migration_items.json"),
            json_path.join("recipes").join("**").join("*.json"),
            json_path.join("requirements").join("**").join("*.json"),
            json_path.join("vehicleparts").join("**").join("*.json"),
        ];
        patterns
            .iter()
            .map(|pattern| {
                pattern
                    .as_path()
                    .to_str()
                    .expect("Path pattern should be valid UTF-8")
            })
            .flat_map(|pattern| {
                println!("Searching {pattern} for info files");
                glob(pattern).expect("Glob pattern should match some readable paths")
            })
            .map(|json_path_result| json_path_result.expect("JSON path should be valid"))
            .collect::<Vec<_>>()
    }

    fn literals() -> HashMap<TypeId, HashMap<String, serde_json::Map<String, serde_json::Value>>> {
        let mut literals = HashMap::default();
        for type_ids in TypeId::all() {
            for type_id in *type_ids {
                literals.insert(type_id.clone(), HashMap::default());
            }
        }

        let mut file_count = 0;
        let mut info_count = 0;
        for json_path in Self::literals_paths() {
            if json_path.ends_with("default_blacklist.json")
                || json_path.ends_with("dreams.json")
                || json_path.ends_with("effect_on_condition.json")
            {
                continue;
            }
            //println!("Parsing {json_path:?}...");
            let file_contents = read_to_string(&json_path)
                .unwrap_or_else(|_| panic!("Could not read {}", json_path.display()));
            let contents = serde_json::from_str::<Vec<serde_json::Map<String, serde_json::Value>>>(
                file_contents.as_str(),
            );
            let contents =
                contents.expect("The file should be valid JSON, containing a list of objects");
            file_count += 1;

            for content in contents {
                let type_ = content.get("type").expect("'type' key should be present");
                let type_ = TypeId::get(type_.as_str().expect("'type' should have string value"));
                if TypeId::UNUSED.contains(type_) || content.get("from_variant").is_some() {
                    continue; // TODO
                }

                let mut ids = Vec::new();
                match id_value(&content, &json_path) {
                    serde_json::Value::String(id) => {
                        ids.push(String::from(id.as_str()));
                    }
                    serde_json::Value::Array(a) => {
                        for id in a {
                            ids.push(String::from(
                                id.as_str().expect("Id should have a string value"),
                            ));
                        }
                    }
                    _ => {
                        unreachable!();
                    }
                }
                //println!("Info abount {:?} > {:?}", &type_, &ids);
                let by_type = literals
                    .get_mut(&type_.clone())
                    .expect("All types should be present");
                for id in ids {
                    if by_type.get(&id).is_some() {
                        eprintln!(
                            "Duplicate usage of id {:?} in {:?} detected. One will be ignored.",
                            &id, &type_
                        );
                    }
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
                                aliases.push(String::from(id.as_str().expect("")));
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
        assert!(!literals.is_empty(), "Some info should have been found");
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
            'enricheds: for (object_id, literal) in literal_entry {
                if literal.contains_key("abstract") {
                    continue;
                }
                //println!("{:?}", &object_id);
                let mut enriched = literal.clone();
                let mut ancestors = vec![object_id.clone()];
                while let Some(copy_from) = enriched.remove("copy-from") {
                    //println!("Copy from {:?}", &copy_from);
                    let copy_from = copy_from
                        .as_str()
                        .expect("'copy-from' should have a string value");
                    ancestors.push(String::from(copy_from));
                    assert!(ancestors.len() < 10, "{ancestors:?}");
                    let literals = &literals;
                    let copy_from = if let Some(found) = literal_entry.get(copy_from) {
                        found
                    } else {
                        let mut other_types = literals
                            .into_iter()
                            .filter_map(|(_, literal_entry)| literal_entry.get(copy_from));
                        let Some(single) = other_types.next() else {
                            eprintln!(
                                "copy-from {copy_from:?} not found for ({:?}) {:?}",
                                &type_id, &copy_from
                            );
                            continue 'enricheds;
                        };
                        if other_types.next().is_some() {
                            eprintln!(
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

                enriched.remove("id");
                enriched.remove("from");
                enriched.remove("copy-from");

                if TypeId::MIGRATION.contains(type_id) {
                    // new_id -> replace <- to
                    if let Some(new_id) = enriched.remove("new_id") {
                        assert!(
                            !enriched.contains_key("to"),
                            "'to' and 'new_id' can not be combined"
                        );
                        let replace = enriched.entry("replace");
                        assert!(
                            matches!(replace, Entry::Vacant { .. }),
                            "'replace' and 'new_id' can not be combined"
                        );
                        replace.or_insert(new_id);
                    } else if let Some(to) = enriched.remove("to") {
                        assert!(
                            !enriched.contains_key("new_id"),
                            "'to' a..   ..nd 'new_id' can not be combined"
                        );
                        let replace = enriched.entry("replace");
                        assert!(
                            matches!(replace, Entry::Vacant { .. }),
                            "'to' and 'replace' can not be combined"
                        );
                        replace.or_insert(to);
                    }
                }

                enriched_of_type.insert(ObjectId::new(object_id), enriched);
            }
        }
        enricheds
    }

    pub(crate) fn load() -> Self {
        let start = Instant::now();

        let mut enricheds = Self::enricheds();
        let mut this = Self {
            ammos: Self::extract(&mut enricheds, TypeId::AMMO),
            bionic_items: Self::extract(&mut enricheds, TypeId::BIONIC_ITEM),
            books: Self::extract(&mut enricheds, TypeId::BOOK),
            characters: Self::extract(&mut enricheds, TypeId::CHARACTER),
            clothings: Self::extract(&mut enricheds, TypeId::CLOTHING),
            comestibles: Self::extract(&mut enricheds, TypeId::COMESTIBLE),
            common_item_info: HashMap::default(),
            engines: Self::extract(&mut enricheds, TypeId::ENGINE),
            fields: Self::extract(&mut enricheds, TypeId::FIELD),
            furniture: Self::extract(&mut enricheds, TypeId::FURNITURE),
            genenric_items: Self::extract(&mut enricheds, TypeId::GENERIC_ITEM),
            guns: Self::extract(&mut enricheds, TypeId::GUN),
            gunmods: Self::extract(&mut enricheds, TypeId::GUNMOD),
            item_groups: Self::extract(&mut enricheds, TypeId::ITEM_GROUP),
            magazines: Self::extract(&mut enricheds, TypeId::MAGAZINE),
            migrations: Self::extract(&mut enricheds, TypeId::MIGRATION),
            pet_armors: Self::extract(&mut enricheds, TypeId::PET_ARMOR),
            qualities: Self::extract(&mut enricheds, TypeId::TOOL_QUALITY),
            recipes: Self::extract(&mut enricheds, TypeId::RECIPE),
            requirements: Self::extract(&mut enricheds, TypeId::REQUIREMENT),
            terrain: Self::extract(&mut enricheds, TypeId::TERRAIN),
            tools: Self::extract(&mut enricheds, TypeId::TOOL),
            tool_clothings: Self::extract(&mut enricheds, TypeId::TOOL_CLOTHING),
            toolmods: Self::extract(&mut enricheds, TypeId::TOOLMOD),
            vehicle_parts: Self::extract(&mut enricheds, TypeId::VEHICLE_PART),
            wheels: Self::extract(&mut enricheds, TypeId::WHEEL),
            zone_levels: Self::extract(&mut enricheds, TypeId::OVERMAP),
        };

        this.characters
            .insert(ObjectId::new("human"), default_human());

        for (id, value) in &this.ammos {
            this.common_item_info
                .insert(id.clone(), value.common.clone());
        }
        for (id, value) in &this.bionic_items {
            this.common_item_info
                .insert(id.clone(), value.common.clone());
        }
        for (id, value) in &this.books {
            this.common_item_info
                .insert(id.clone(), value.common.clone());
        }
        for (id, value) in &this.clothings {
            this.common_item_info
                .insert(id.clone(), value.common.clone());
        }
        for (id, value) in &this.comestibles {
            this.common_item_info
                .insert(id.clone(), value.common.clone());
        }
        for (id, value) in &this.engines {
            this.common_item_info
                .insert(id.clone(), value.common.clone());
        }
        for (id, value) in &this.genenric_items {
            this.common_item_info
                .insert(id.clone(), value.common.clone());
        }
        for (id, value) in &this.guns {
            this.common_item_info
                .insert(id.clone(), value.common.clone());
        }
        for (id, value) in &this.gunmods {
            this.common_item_info
                .insert(id.clone(), value.common.clone());
        }
        for (id, value) in &this.magazines {
            this.common_item_info
                .insert(id.clone(), value.common.clone());
        }
        for (id, value) in &this.pet_armors {
            this.common_item_info
                .insert(id.clone(), value.common.clone());
        }
        for (id, value) in &this.tools {
            this.common_item_info
                .insert(id.clone(), value.common.clone());
        }
        for (id, value) in &this.tool_clothings {
            this.common_item_info
                .insert(id.clone(), value.clothing.common.clone());
        }
        for (id, value) in &this.toolmods {
            this.common_item_info
                .insert(id.clone(), value.common.clone());
        }
        for (id, value) in &this.wheels {
            this.common_item_info
                .insert(id.clone(), value.common.clone());
        }

        let duration = start.elapsed();
        println!("The creation of Infos took {duration:?}");

        this
    }

    pub(crate) fn try_character<'a>(&'a self, id: &'a ObjectId) -> Option<&'a CharacterInfo> {
        self.try_get(&self.characters, id)
    }

    pub(crate) fn character<'a>(&'a self, id: &'a ObjectId) -> &'a CharacterInfo {
        self.get(&self.characters, id)
    }

    pub(crate) fn try_common_item_info<'a>(
        &'a self,
        id: &'a ObjectId,
    ) -> Option<&'a CommonItemInfo> {
        self.try_get(&self.common_item_info, id)
    }

    pub(crate) fn try_field<'a>(&'a self, id: &'a ObjectId) -> Option<&'a FieldInfo> {
        self.try_get(&self.fields, id)
    }

    pub(crate) fn try_furniture<'a>(&'a self, id: &'a ObjectId) -> Option<&'a FurnitureInfo> {
        self.try_get(&self.furniture, id)
    }

    pub(crate) fn try_item_group<'a>(&'a self, id: &'a ObjectId) -> Option<&'a ItemGroup> {
        self.try_get(&self.item_groups, id)
    }

    #[expect(unused)]
    pub(crate) fn try_quality<'a>(&'a self, id: &'a ObjectId) -> Option<&'a Quality> {
        self.try_get(&self.qualities, id)
    }

    pub(crate) fn quality<'a>(&'a self, id: &'a ObjectId) -> &'a Quality {
        self.get(&self.qualities, id)
    }

    #[expect(unused)]
    pub(crate) fn try_recipe<'a>(&'a self, id: &'a ObjectId) -> Option<&'a Recipe> {
        self.try_get(&self.recipes, id)
    }

    pub(crate) fn recipe<'a>(&'a self, id: &'a ObjectId) -> &'a Recipe {
        self.get(&self.recipes, id)
    }

    pub(crate) fn try_requirement<'a>(&'a self, id: &'a ObjectId) -> Option<&'a Requirement> {
        self.try_get(&self.requirements, id)
    }

    pub(crate) fn requirement<'a>(&'a self, id: &'a ObjectId) -> &'a Requirement {
        self.get(&self.requirements, id)
    }

    pub(crate) fn try_terrain<'a>(&'a self, id: &'a ObjectId) -> Option<&'a TerrainInfo> {
        self.try_get(&self.terrain, id)
    }

    pub(crate) fn terrain<'a>(&'a self, id: &'a ObjectId) -> &'a TerrainInfo {
        self.get(&self.terrain, id)
    }

    pub(crate) fn try_vehicle_part<'a>(&'a self, id: &'a ObjectId) -> Option<&'a VehiclePartInfo> {
        self.try_get(&self.vehicle_parts, id)
    }

    pub(crate) fn try_zone_level<'a>(&'a self, id: &'a ObjectId) -> Option<&'a OvermapInfo> {
        self.try_get(&self.zone_levels, id)
    }

    fn get<'a, T>(&'a self, map: &'a HashMap<ObjectId, T>, id: &'a ObjectId) -> &'a T {
        self.try_get(map, id)
            .unwrap_or_else(|| panic!("{id:?} ({}) should be known", type_name::<T>()))
    }

    fn try_get<'a, T>(&'a self, map: &'a HashMap<ObjectId, T>, id: &'a ObjectId) -> Option<&'a T> {
        map.get(self.maybe_migrated(id))
    }

    fn maybe_migrated<'a>(&'a self, id: &'a ObjectId) -> &'a ObjectId {
        self.migrations.get(id).map_or(id, |m| &m.replace)
    }

    fn looks_like(&self, definition: &ObjectDefinition) -> Option<ObjectId> {
        match definition.category {
            ObjectCategory::Character => self
                .try_character(&definition.id)
                .and_then(|o| o.looks_like.clone()),
            ObjectCategory::Item => self
                .try_common_item_info(&definition.id)
                .and_then(|o| o.looks_like.clone()),
            ObjectCategory::Field => self
                .try_field(&definition.id)
                .and_then(|o| o.looks_like.clone()),
            ObjectCategory::Furniture => self
                .try_furniture(&definition.id)
                .and_then(|o| o.looks_like.clone()),
            ObjectCategory::Terrain => self
                .try_terrain(&definition.id)
                .and_then(|o| o.looks_like.clone()),
            ObjectCategory::VehiclePart => self
                .try_vehicle_part(&definition.id)
                .and_then(|o| o.looks_like.clone()),
            ObjectCategory::ZoneLevel => self
                .try_zone_level(&definition.id)
                .and_then(|o| o.looks_like.clone()),
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
            current_definition_ref.id.prefix("vp_"),
            current_definition_ref.id.prefix("vp_").suffix("_cover"),
        ];

        while let Some(other) = self.looks_like(current_definition_ref) {
            if variants.contains(&other) {
                //eprintln!("Variants {:?} already contains {:?}", &variants, &other); // TODO
                break;
            }
            variants.push(other.suffix("_season_summer"));
            variants.push(other.clone());
            variants.push(other.prefix("vp_"));
            variants.push(other.prefix("vp_").suffix("_cover"));
            current_definition = ObjectDefinition {
                category: definition.category.clone(),
                id: other.clone(),
            };
            current_definition_ref = &current_definition;
        }
        variants
    }

    pub(crate) fn qualities(&self) -> impl Iterator<Item = ObjectId> + '_ {
        self.qualities.keys().cloned()
    }

    pub(crate) fn recipes(&self) -> impl Iterator<Item = (&ObjectId, &Recipe)> {
        self.recipes.iter()
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
                //println!("{:#?}", &object_properties);
                let info: T = serde_json::from_value(serde_json::Value::Object(object_properties))
                    .unwrap_or_else(|e| panic!("{:?} {:?}", &id, &e));
                result.insert(id, info);
            }
        }

        result
    }

    pub(crate) fn to_components(&self, using: &Using) -> Vec<Vec<Alternative>> {
        if using.kind == UsingKind::Components {
            self.requirement(&using.requirement)
                .components
                .clone()
                .into_iter()
                .map(|component| {
                    component
                        .into_iter()
                        .map(|mut alternative| {
                            *match alternative {
                                Alternative::Item {
                                    ref mut required, ..
                                } => required,
                                Alternative::Requirement { ref mut factor, .. } => factor,
                            } *= using.factor;
                            alternative
                        })
                        .collect()
                })
                .collect()
        } else {
            vec![vec![Alternative::Requirement {
                requirement: using.requirement.clone(),
                factor: using.factor,
            }]]
        }
    }
}

fn default_human() -> CharacterInfo {
    CharacterInfo {
        name: ItemName::from(CddaItemName::Simple(Arc::from("Human"))),
        default_faction: Arc::from("human"),
        looks_like: Some(ObjectId::new("overlay_male_mutation_SKIN_TAN")),
        volume: Some(Volume::from("80 l")),
        mass: Some(Mass::from("80 kg")),
        hp: Some(100),
        speed: 100,
        melee_dice: 2,
        melee_dice_sides: 4,
        flags: Flags::default(),
        extra: HashMap::default(),
    }
}

impl AsyncNew<Self> for Infos {
    async fn async_new() -> Self {
        Self::load()
    }
}

fn id_value<'a>(
    content: &'a serde_json::Map<String, serde_json::Value>,
    json_path: &'a PathBuf,
) -> &'a serde_json::Value {
    if content
        .get("type")
        .is_some_and(|v| v.as_str() == Some("recipe"))
    {
        assert_eq!(content.get("id"), None, "No 'id' field allowed");
        assert_eq!(content.get("from"), None, "No 'from' field allowed");

        if let Some(suffix) = content.get("id_suffix") {
            let composite = serde_json::Value::String(
                content
                    .get("result")
                    .or_else(|| content.get("copy-from"))
                    .unwrap_or_else(|| {
                        panic!(
                            "A recipe with an id suffix should have a 'result', or a 'copy-from' field: {content:?}"
                        )
                    })
                    .as_str()
                    .expect("'result' should be a string field")
                    .to_owned()
                    + " " + suffix
                        .as_str()
                        .expect("'id_suffix' should be a string field"),
            );
            return Box::leak(Box::new(composite));
        }

        return content
            .get("abstract")
            .or_else(|| content.get("result"))
            .unwrap_or_else(|| {
                panic!("A recipe should have an 'abstract' field, or a 'result' field: {content:?}")
            });
    }

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
    assert_eq!(count, 1, "Not one of id, abstract, or from for json with type {:?} and keys  ({:?}) from {json_path:?}",
             content.get("type"), content.keys().collect::<Vec<_>>());
    content
        .get("id")
        .or_else(|| content.get("abstract"))
        .or_else(|| content.get("from"))
        .expect("id, abstract, or from")
}
