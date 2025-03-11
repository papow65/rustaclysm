use crate::gameplay::{ObjectCategory, ObjectDefinition, TypeId, cdda::Error};
use crate::util::{AssetPaths, AsyncNew};
use bevy::prelude::{Component, Resource};
use bevy::utils::HashMap;
use cdda_json_files::{
    Alternative, Ammo, BionicItem, Book, CddaItemName, CharacterInfo, Clothing, Comestible,
    CommonItemInfo, Engine, FieldInfo, Flags, FurnitureInfo, GenericItem, Gun, Gunmod, ItemGroup,
    ItemMigration, ItemName, ItemWithCommonInfo, Magazine, ObjectId, OvermapInfo, PetArmor,
    Quality, Recipe, Requirement, Submap, TerrainInfo, Tool, ToolClothing, Toolmod, Using,
    UsingKind, VehiclePartInfo, VehiclePartMigration, Wheel,
};
use glob::glob;
use serde::de::DeserializeOwned;
use std::{
    any::type_name, fs::read_to_string, ops::Deref, path::PathBuf, sync::Arc, time::Instant,
};
use units::{Mass, Volume};

#[derive(Debug, Component)]
pub(crate) struct Info<T>(Arc<T>);

impl<T> Info<T> {
    pub(crate) const fn new(arc: Arc<T>) -> Self {
        Self(arc)
    }
}

impl<T> AsRef<T> for Info<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Clone for Info<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Deref for Info<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

pub(crate) struct InfoMap<T> {
    map: HashMap<ObjectId, Arc<T>>,
}

impl<T: DeserializeOwned + 'static> InfoMap<T> {
    fn new(
        all: &mut HashMap<TypeId, HashMap<ObjectId, serde_json::Map<String, serde_json::Value>>>,
        type_ids: &[TypeId],
    ) -> Self {
        let mut map = HashMap::default();
        for type_id in type_ids {
            let objects = all
                .remove(type_id)
                .unwrap_or_else(|| panic!("Type {type_id:?} not found"));
            for (id, object_properties) in objects {
                //println!("{:#?}", &object_properties);
                match serde_json::from_value::<T>(serde_json::Value::Object(object_properties)) {
                    Ok(info) => {
                        map.insert(id, Arc::new(info));
                    }
                    Err(error) => {
                        eprintln!(
                            "Failed loading json for {:?} {:?}: {error:#?}",
                            std::any::TypeId::of::<T>(),
                            &id
                        );
                    }
                }
            }
        }

        Self { map }
    }

    pub(crate) fn get(&self, id: &ObjectId) -> Result<&Arc<T>, Error> {
        self.map.get(id).ok_or_else(|| Error::UnknownObject {
            _id: id.clone(),
            _type: type_name::<T>(),
        })
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &Arc<T>> {
        self.map.values()
    }
}

impl InfoMap<CommonItemInfo> {
    fn link_common_items(&self, qualities: &InfoMap<Quality>) {
        for common_item_info in self.map.values() {
            common_item_info
                .qualities
                .finalize(&qualities.map, "quality");
        }
    }
}

impl InfoMap<FurnitureInfo> {
    fn link_furniture(&self, common_item_infos: &InfoMap<CommonItemInfo>) {
        for furniture_info in self.map.values() {
            furniture_info
                .crafting_pseudo_item
                .finalize(&common_item_infos.map, "pseudo item");
            if let Some(bash) = &furniture_info.bash {
                bash.terrain
                    .finalize(&HashMap::default(), "terrain for bashed furniture");
                bash.furniture.finalize(&self.map, "bashed furniture");
            }
        }
    }
}

impl InfoMap<Recipe> {
    fn link_recipes(
        &self,
        qualities: &InfoMap<Quality>,
        common_item_infos: &InfoMap<CommonItemInfo>,
    ) {
        for recipe in self.map.values() {
            for required_quality in &recipe.qualities.0 {
                required_quality
                    .quality
                    .finalize(&qualities.map, "required quality for recipe");
            }

            recipe.result.finalize(&common_item_infos.map, "recipe");
        }
    }
}

impl InfoMap<Requirement> {
    fn link_requirements(&self, qualities: &InfoMap<Quality>) {
        for requirement in self.map.values() {
            for required_quality in &requirement.qualities.0 {
                required_quality
                    .quality
                    .finalize(&qualities.map, "required quality for requirement");
            }
        }
    }
}

impl InfoMap<TerrainInfo> {
    fn fix_and_link_terrain(&mut self, furniture: &InfoMap<FurnitureInfo>) {
        if self.map.remove(&ObjectId::new("t_null")).is_some() {
            eprintln!("The terrain t_null was not expected to be present");
        }

        for terrain_info in self.map.values() {
            terrain_info.open.finalize(&self.map, "open terrain");
            terrain_info.close.finalize(&self.map, "closed terrain");
            if let Some(bash) = &terrain_info.bash {
                bash.terrain.finalize(&self.map, "bashed terrain");
                if bash.terrain.get().is_none() {
                    eprintln!("No bashed terrain set for {:?}", terrain_info.id);
                }

                bash.furniture
                    .finalize(&furniture.map, "furniture for bashed terrain");
            }
        }
    }
}

impl InfoMap<VehiclePartInfo> {
    fn add_vehicle_part_migrations(
        &mut self,
        vehicle_part_migrations: &HashMap<ObjectId, Arc<VehiclePartMigration>>,
    ) {
        // TODO Make this recursive
        for (migration_from, migration) in vehicle_part_migrations {
            if let Ok(new) = self.get(migration_from).cloned() {
                self.map.insert(migration.replace.clone(), new);
            }
        }
    }
}

struct ItemInfoMapLoader<'a> {
    enriched_json_infos:
        &'a mut HashMap<TypeId, HashMap<ObjectId, serde_json::Map<String, serde_json::Value>>>,
    item_migrations: HashMap<ObjectId, Arc<ItemMigration>>,
    common_item_infos: &'a mut InfoMap<CommonItemInfo>,
}

impl ItemInfoMapLoader<'_> {
    fn item_extract<T>(&mut self, type_ids: &[TypeId]) -> InfoMap<T>
    where
        T: DeserializeOwned + ItemWithCommonInfo + 'static,
    {
        let mut items = InfoMap::<T>::new(self.enriched_json_infos, type_ids);

        // TODO Make this recursive
        for (migration_from, migration) in &self.item_migrations {
            if let Ok(new) = items.get(migration_from).cloned() {
                items.map.insert(migration.replace.clone(), new);
            }
        }

        for (id, item_info) in &mut items.map {
            self.common_item_infos
                .map
                .insert(id.clone(), item_info.common());
        }

        items
    }
}

#[derive(Resource)]
pub(crate) struct Infos {
    #[expect(unused)]
    pub(crate) ammos: InfoMap<Ammo>,

    #[expect(unused)]
    pub(crate) bionic_items: InfoMap<BionicItem>,

    #[expect(unused)]
    pub(crate) books: InfoMap<Book>,

    pub(crate) characters: InfoMap<CharacterInfo>,

    #[expect(unused)]
    pub(crate) clothings: InfoMap<Clothing>,

    #[expect(unused)]
    pub(crate) comestibles: InfoMap<Comestible>,

    pub(crate) common_item_infos: InfoMap<CommonItemInfo>,

    #[expect(unused)]
    pub(crate) engines: InfoMap<Engine>,

    pub(crate) fields: InfoMap<FieldInfo>,
    pub(crate) furniture: InfoMap<FurnitureInfo>,

    #[expect(unused)]
    pub(crate) genenric_items: InfoMap<GenericItem>,

    #[expect(unused)]
    pub(crate) guns: InfoMap<Gun>,

    #[expect(unused)]
    pub(crate) gunmods: InfoMap<Gunmod>,

    pub(crate) item_groups: InfoMap<ItemGroup>,
    pub(crate) magazines: InfoMap<Magazine>,

    #[expect(unused)]
    pub(crate) pet_armors: InfoMap<PetArmor>,

    #[expect(unused)]
    pub(crate) qualities: InfoMap<Quality>,

    pub(crate) recipes: InfoMap<Recipe>,
    pub(crate) requirements: InfoMap<Requirement>,
    pub(crate) terrain: InfoMap<TerrainInfo>,

    #[expect(unused)]
    pub(crate) tools: InfoMap<Tool>,

    #[expect(unused)]
    pub(crate) tool_clothings: InfoMap<ToolClothing>,

    #[expect(unused)]
    pub(crate) toolmods: InfoMap<Toolmod>,

    pub(crate) vehicle_parts: InfoMap<VehiclePartInfo>,

    #[expect(unused)]
    pub(crate) wheels: InfoMap<Wheel>,

    pub(crate) zone_levels: InfoMap<OvermapInfo>,
}

impl Infos {
    fn literals_paths() -> impl Iterator<Item = PathBuf> {
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
            .into_iter()
            .flat_map(|pattern| {
                let pattern = pattern
                    .as_path()
                    .to_str()
                    .expect("Path pattern should be valid UTF-8");
                println!("Searching {pattern} for info files");
                glob(pattern).expect("Glob pattern should match some readable paths")
            })
            .map(|json_path_result| json_path_result.expect("JSON path should be valid"))
            .filter(|path| {
                !path.ends_with("default_blacklist.json")
                    && !path.ends_with("dreams.json")
                    && !path.ends_with("effect_on_condition.json")
            })
    }

    /// [`TypeId`] -> [`ObjectId`] -> property name -> property value
    fn literal_json_infos()
    -> HashMap<TypeId, HashMap<ObjectId, serde_json::Map<String, serde_json::Value>>> {
        let mut literals = HashMap::default();
        for type_ids in TypeId::all() {
            for type_id in *type_ids {
                literals.insert(type_id.clone(), HashMap::default());
            }
        }

        let mut file_count = 0;
        let mut info_count = 0;
        for json_path in Self::literals_paths() {
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
                        ids.push(ObjectId::new(id.as_str()));
                    }
                    serde_json::Value::Array(a) => {
                        for id in a {
                            ids.push(ObjectId::new(
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
                            aliases.push(ObjectId::new(id.as_str()));
                        }
                        serde_json::Value::Array(a) => {
                            for id in a {
                                aliases.push(ObjectId::new(id.as_str().expect("")));
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

    /// [`TypeId`] -> [`ObjectId`] -> property name -> property value
    fn enriched_json_infos()
    -> HashMap<TypeId, HashMap<ObjectId, serde_json::Map<String, serde_json::Value>>> {
        let mut enriched_json_infos = HashMap::default();
        let literals = &Self::literal_json_infos();
        for (type_id, literal_entry) in literals {
            let enriched_of_type = enriched_json_infos
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
                    let copy_from = ObjectId::new(
                        copy_from
                            .as_str()
                            .expect("'copy-from' should have a string value"),
                    );
                    ancestors.push(copy_from.clone());
                    assert!(ancestors.len() < 10, "{ancestors:?}");
                    let copy_from = if let Some(found) = literal_entry.get(&copy_from) {
                        found
                    } else {
                        let mut other_types = literals
                            .into_iter()
                            .filter_map(|(_, literal_entry)| literal_entry.get(&copy_from));
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

                enriched.remove("copy-from");

                if TypeId::RECIPE.contains(type_id) {
                    set_recipe_id(&mut enriched);
                } else if !TypeId::FURNITURE.contains(type_id)
                    && !TypeId::TERRAIN.contains(type_id)
                    && !TypeId::TOOL_QUALITY.contains(type_id)
                {
                    enriched.remove("id");
                }

                enriched_of_type.insert(object_id.clone(), enriched);
            }
        }
        enriched_json_infos
    }

    fn load() -> Self {
        let start = Instant::now();

        let mut enriched_json_infos = Self::enriched_json_infos();

        let mut common_item_infos = InfoMap {
            map: HashMap::default(),
        };
        let qualities = InfoMap::new(&mut enriched_json_infos, TypeId::TOOL_QUALITY);

        let item_migrations = InfoMap::new(&mut enriched_json_infos, TypeId::ITEM_MIGRATION).map;
        let mut item_loader = ItemInfoMapLoader {
            enriched_json_infos: &mut enriched_json_infos,
            item_migrations,
            common_item_infos: &mut common_item_infos,
        };
        let ammos = item_loader.item_extract(TypeId::AMMO);
        let bionic_items = item_loader.item_extract(TypeId::BIONIC_ITEM);
        let books = item_loader.item_extract(TypeId::BOOK);
        let clothings = item_loader.item_extract(TypeId::CLOTHING);
        let comestibles = item_loader.item_extract(TypeId::COMESTIBLE);
        let engines = item_loader.item_extract(TypeId::ENGINE);
        let genenric_items = item_loader.item_extract(TypeId::GENERIC_ITEM);
        let guns = item_loader.item_extract(TypeId::GUN);
        let gunmods = item_loader.item_extract(TypeId::GUNMOD);
        let magazines = item_loader.item_extract(TypeId::MAGAZINE);
        let pet_armors = item_loader.item_extract(TypeId::PET_ARMOR);
        let tools = item_loader.item_extract(TypeId::TOOL);
        let tool_clothings = item_loader.item_extract(TypeId::TOOL_CLOTHING);
        let toolmods = item_loader.item_extract(TypeId::TOOLMOD);
        let wheels = item_loader.item_extract(TypeId::WHEEL);
        // item_loader is dropped
        common_item_infos.link_common_items(&qualities);

        let furniture = InfoMap::new(&mut enriched_json_infos, TypeId::FURNITURE);
        furniture.link_furniture(&common_item_infos);
        let mut terrain = InfoMap::new(&mut enriched_json_infos, TypeId::TERRAIN);
        terrain.fix_and_link_terrain(&furniture);
        let requirements = InfoMap::new(&mut enriched_json_infos, TypeId::REQUIREMENT);
        requirements.link_requirements(&qualities);
        let recipes = InfoMap::new(&mut enriched_json_infos, TypeId::RECIPE);
        recipes.link_recipes(&qualities, &common_item_infos);

        let vehicle_part_migrations =
            InfoMap::new(&mut enriched_json_infos, TypeId::VEHICLE_PART_MIGRATION);
        let mut vehicle_parts = InfoMap::new(&mut enriched_json_infos, TypeId::VEHICLE_PART);
        vehicle_parts.add_vehicle_part_migrations(&vehicle_part_migrations.map);

        let mut this = Self {
            ammos,
            bionic_items,
            books,
            characters: InfoMap::new(&mut enriched_json_infos, TypeId::CHARACTER),
            clothings,
            comestibles,
            common_item_infos,
            engines,
            fields: InfoMap::new(&mut enriched_json_infos, TypeId::FIELD),
            furniture,
            genenric_items,
            guns,
            gunmods,
            item_groups: InfoMap::new(&mut enriched_json_infos, TypeId::ITEM_GROUP),
            magazines,
            pet_armors,
            qualities,
            recipes,
            requirements,
            terrain,
            tools,
            tool_clothings,
            toolmods,
            vehicle_parts,
            wheels,
            zone_levels: InfoMap::new(&mut enriched_json_infos, TypeId::OVERMAP),
        };

        for type_id in enriched_json_infos.into_keys() {
            eprintln!("Unprocessed type: {type_id:?}");
        }

        this.characters
            .map
            .insert(ObjectId::new("human"), Arc::new(default_human()));

        let duration = start.elapsed();
        println!("The creation of Infos took {duration:?}");

        this
    }

    fn looks_like(&self, definition: &ObjectDefinition) -> Option<ObjectId> {
        match definition.category {
            ObjectCategory::Character => self
                .characters
                .get(&definition.id)
                .ok()
                .and_then(|o| o.looks_like.clone()),
            ObjectCategory::Item => self
                .common_item_infos
                .get(&definition.id)
                .ok()
                .and_then(|o| o.looks_like.clone()),
            ObjectCategory::Field => self
                .fields
                .get(&definition.id)
                .ok()
                .and_then(|o| o.looks_like.clone()),
            ObjectCategory::Furniture => self
                .furniture
                .get(&definition.id)
                .ok()
                .and_then(|o| o.looks_like.clone()),
            ObjectCategory::Terrain => self
                .terrain
                .get(&definition.id)
                .ok()
                .and_then(|o| o.looks_like.clone()),
            ObjectCategory::VehiclePart => self
                .vehicle_parts
                .get(&definition.id)
                .ok()
                .and_then(|o| o.looks_like.clone()),
            ObjectCategory::ZoneLevel => self
                .zone_levels
                .get(&definition.id)
                .ok()
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

    pub(crate) fn to_components(&self, using: &Using) -> Result<Vec<Vec<Alternative>>, Error> {
        Ok(if using.kind == UsingKind::Components {
            self.requirements
                .get(&using.requirement)?
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
        })
    }

    pub(crate) fn link_submap(&self, submap: &Submap) {
        if submap.linked.set(()).is_err() {
            return;
        }

        for terrain_repetition in &submap.terrain.0 {
            terrain_repetition
                .as_amount()
                .obj
                .finalize(&self.terrain.map, "submap terrain");
        }
        for furniture_at in &submap.furniture {
            furniture_at
                .obj
                .finalize(&self.furniture.map, "submap furniture");
        }
    }
}

fn set_recipe_id(enriched: &mut serde_json::Map<String, serde_json::Value>) {
    if let Some(recipe_id) = enriched.get("id") {
        eprintln!("Recipe should not have an id: {recipe_id:?}");
    } else if let Some(result) = enriched.get("result").cloned() {
        if let Some(result_str) = result.as_str() {
            let id = String::from(result_str)
                + enriched
                    .get("id_suffix")
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
            enriched
                .entry("id")
                .or_insert(serde_json::Value::String(id));
        } else {
            panic!("Recipe result should be a string: {result:#?}");
        }
    } else {
        panic!("Recipe should have a result: {enriched:#?}");
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
    assert_eq!(
        count,
        1,
        "Not one of id, abstract, or from for json with type {:?} and keys  ({:?}) from {json_path:?}",
        content.get("type"),
        content.keys().collect::<Vec<_>>()
    );
    content
        .get("id")
        .or_else(|| content.get("abstract"))
        .or_else(|| content.get("from"))
        .expect("id, abstract, or from")
}
