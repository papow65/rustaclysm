use crate::gameplay::cdda::info::info_map::{InfoMap, ItemInfoMapLoader};
use crate::gameplay::cdda::info::parsed_json::ParsedJson;
use crate::gameplay::{ObjectCategory, ObjectDefinition, TypeId, cdda::Error};
use crate::util::AsyncNew;
use bevy::prelude::Resource;
use bevy::utils::HashMap;
use cdda_json_files::{
    Alternative, Ammo, BionicItem, Book, CddaItem, CddaItemName, CharacterInfo, Clothing,
    Comestible, CommonItemInfo, Engine, FieldInfo, Flags, FurnitureInfo, GenericItem, Gun, Gunmod,
    ItemGroup, ItemName, Magazine, ObjectId, Overmap, OvermapInfo, PetArmor, Quality, Recipe,
    RequiredLinkedLater, Requirement, Submap, TerrainInfo, Tool, ToolClothing, Toolmod, Using,
    UsingKind, VehiclePartInfo, Wheel,
};
use std::{sync::Arc, time::Instant};
use units::{Mass, Volume};

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
    fn load() -> Self {
        let start = Instant::now();

        let mut enriched_json_infos = ParsedJson::enriched();

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
        terrain.fix_and_link_terrain(&furniture, &common_item_infos);
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

    pub(crate) fn link_overmap(&self, overmap: &Overmap) {
        if overmap.linked.set(()).is_err() {
            return;
        }

        for (_, monster) in &overmap.monster_map.0 {
            monster
                .info
                .finalize(&self.characters.map, "overmap monster");
        }
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

        for fields_at in &submap.fields.0 {
            for field in &fields_at.obj.0 {
                field.field_info.finalize(&self.fields.map, "submap field");
            }
        }

        for character in &submap.spawns {
            self.link_character(&character.info, "submap spawn");
        }

        for items_at in &submap.items.0 {
            for item in &items_at.obj {
                self.link_item(&item.as_amount().obj);
            }
        }
    }

    fn link_item(&self, item: &CddaItem) {
        item.item_info
            .finalize(&self.common_item_infos.map, "submap item");
        item.corpse
            .finalize(&self.characters.map, "submap item corpse");

        if let Some(contents) = &item.contents {
            for pocket in &contents.contents {
                for content in &pocket.contents {
                    self.link_item(content);
                }
            }
        }
    }

    pub(crate) fn link_character(
        &self,
        character: &RequiredLinkedLater<CharacterInfo>,
        err_description: &str,
    ) {
        character.finalize(&self.characters.map, err_description);
    }
}

impl AsyncNew<Self> for Infos {
    async fn async_new() -> Self {
        Self::load()
    }
}

fn default_human() -> CharacterInfo {
    CharacterInfo {
        id: ObjectId::new("human"),
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
