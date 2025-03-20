use crate::gameplay::cdda::info::info_map::{InfoMap, ItemInfoMapLoader};
use crate::gameplay::cdda::info::parsed_json::ParsedJson;
use crate::gameplay::{ObjectCategory, TypeId, cdda::Error};
use crate::util::AsyncNew;
use bevy::prelude::{Resource, debug, error, info};
use bevy::utils::HashMap;
use cdda_json_files::{
    Alternative, Ammo, BionicItem, Book, CddaItem, CddaItemName, CharacterInfo, Clothing,
    Comestible, CommonItemInfo, Engine, FieldInfo, Flags, FurnitureInfo, GenericItem, Gun, Gunmod,
    InfoId, ItemGroup, ItemName, Magazine, Overmap, OvermapTerrainInfo, PetArmor, Quality, Recipe,
    RequiredLinkedLater, Requirement, Submap, TerrainInfo, Tool, ToolClothing, Toolmod,
    UntypedInfoId, Using, UsingKind, VehiclePartInfo, VehiclePartMigration, Wheel,
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

    #[expect(unused)]
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

    pub(crate) vehicle_parts_info: InfoMap<VehiclePartInfo>,

    #[expect(unused)]
    pub(crate) wheels: InfoMap<Wheel>,

    pub(crate) zone_levels: InfoMap<OvermapTerrainInfo>,
}

impl Infos {
    fn load() -> Self {
        let start = Instant::now();

        let mut enriched_json_infos = ParsedJson::enriched();
        debug!(
            "Collected {} enriched categories",
            enriched_json_infos.len()
        );
        for (type_id, values) in &enriched_json_infos {
            if !values.is_empty() {
                debug!("Collected {} {type_id:?} entries", values.len());
            }
        }

        let qualities = InfoMap::new(&mut enriched_json_infos, TypeId::ToolQuality);

        let item_migrations = InfoMap::new(&mut enriched_json_infos, TypeId::ItemMigration).map;
        let mut common_item_infos = InfoMap {
            map: HashMap::default(),
        };
        let mut item_loader = ItemInfoMapLoader {
            enriched_json_infos: &mut enriched_json_infos,
            item_migrations,
            common_item_infos: &mut common_item_infos,
        };
        let ammos = item_loader.item_extract(TypeId::Ammo);
        let bionic_items = item_loader.item_extract(TypeId::BionicItem);
        let books = item_loader.item_extract(TypeId::Book);
        let clothings = item_loader.item_extract(TypeId::Clothing);
        let comestibles = item_loader.item_extract(TypeId::Comestible);
        let engines = item_loader.item_extract(TypeId::Engine);
        let genenric_items = item_loader.item_extract(TypeId::GenericItem);
        let guns = item_loader.item_extract(TypeId::Gun);
        let gunmods = item_loader.item_extract(TypeId::GunMod);
        let magazines = item_loader.item_extract(TypeId::Magazine);
        let pet_armors = item_loader.item_extract(TypeId::PetArmor);
        let tools = item_loader.item_extract(TypeId::Tool);
        let tool_clothings = item_loader.item_extract(TypeId::ToolClothing);
        let toolmods = item_loader.item_extract(TypeId::ToolMod);
        let wheels = item_loader.item_extract(TypeId::Wheel);
        // item_loader is dropped

        debug!("Collected {} common items", common_item_infos.map.len());
        common_item_infos.link_common_items(&qualities);

        let item_groups = InfoMap::new(&mut enriched_json_infos, TypeId::ItemGroup);

        let furniture = InfoMap::new(&mut enriched_json_infos, TypeId::Furniture);
        furniture.link_furniture(&common_item_infos, &item_groups);
        let mut terrain = InfoMap::new(&mut enriched_json_infos, TypeId::Terrain);
        terrain.fix_and_link_terrain(&furniture, &common_item_infos, &item_groups);
        let requirements = InfoMap::new(&mut enriched_json_infos, TypeId::Requirement);
        requirements.link_requirements(&qualities);
        let recipes = InfoMap::new(&mut enriched_json_infos, TypeId::Recipe);
        recipes.link_recipes(&qualities, &common_item_infos);

        let vehicle_part_migrations = InfoMap::<VehiclePartMigration>::new(
            &mut enriched_json_infos,
            TypeId::VehiclePartMigration,
        );
        let mut vehicle_parts = InfoMap::new(&mut enriched_json_infos, TypeId::VehiclePart);
        vehicle_parts.link_items(&common_item_infos);
        vehicle_parts.add_vehicle_part_migrations(vehicle_part_migrations.map.values());

        let mut this = Self {
            ammos,
            bionic_items,
            books,
            characters: InfoMap::new(&mut enriched_json_infos, TypeId::Character),
            clothings,
            comestibles,
            common_item_infos,
            engines,
            fields: InfoMap::new(&mut enriched_json_infos, TypeId::Field),
            furniture,
            genenric_items,
            guns,
            gunmods,
            item_groups,
            magazines,
            pet_armors,
            qualities,
            recipes,
            requirements,
            terrain,
            tools,
            tool_clothings,
            toolmods,
            vehicle_parts_info: vehicle_parts,
            wheels,
            zone_levels: InfoMap::new(&mut enriched_json_infos, TypeId::OvermapTerrain),
        };

        let mut missing_types = enriched_json_infos.into_keys().map(|type_id| format!("{type_id:?}", )).collect::<Vec<_>>();
        if !missing_types.is_empty() {
            missing_types.sort();
            error!("{} unprocessed types: {}", missing_types.len(), missing_types.join(", "));
        }

        this.characters
            .map
            .insert(InfoId::new("human"), Arc::new(default_human()));

        let duration = start.elapsed();
        info!("The creation of Infos took {duration:?}");

        this
    }

    fn looks_like(
        &self,
        info_id: UntypedInfoId,
        category: ObjectCategory,
    ) -> Option<UntypedInfoId> {
        match category {
            ObjectCategory::Character => self
                .characters
                .get(&info_id.into())
                .ok()
                .and_then(|o| o.looks_like.clone()),
            ObjectCategory::Item => self
                .common_item_infos
                .get(&info_id.into())
                .ok()
                .and_then(|o| o.looks_like.clone()),
            ObjectCategory::Field => self
                .fields
                .get(&info_id.into())
                .ok()
                .and_then(|o| o.looks_like.clone()),
            ObjectCategory::Furniture => self
                .furniture
                .get(&info_id.into())
                .ok()
                .and_then(|o| o.looks_like.clone()),
            ObjectCategory::Terrain => self
                .terrain
                .get(&info_id.into())
                .ok()
                .and_then(|o| o.looks_like.clone()),
            ObjectCategory::VehiclePart => self
                .vehicle_parts_info
                .get(&info_id.into())
                .ok()
                .and_then(|o| o.looks_like.clone()),
            ObjectCategory::ZoneLevel => self
                .zone_levels
                .get(&info_id.into())
                .ok()
                .and_then(|o| o.looks_like.clone()),
            _ => unimplemented!("{info_id:?} {category:?}"),
        }
    }

    pub(crate) fn variants(
        &self,
        info_id: UntypedInfoId,
        category: ObjectCategory,
    ) -> Vec<UntypedInfoId> {
        if category == ObjectCategory::Meta {
            return vec![info_id];
        }

        let mut current_id = if category == ObjectCategory::ZoneLevel {
            info_id.truncate()
        } else {
            info_id
        };
        let mut variants = vec![
            current_id.suffix("_season_summer"),
            current_id.clone(),
            current_id.prefix("vp_"),
            current_id.prefix("vp_").suffix("_cover"),
        ];

        while let Some(other) = self.looks_like(current_id, category) {
            if variants.contains(&other) {
                //trace!("Variants {:?} already contains {:?}", &variants, &other); // TODO
                break;
            }
            variants.push(other.suffix("_season_summer"));
            variants.push(other.clone());
            variants.push(other.prefix("vp_"));
            variants.push(other.prefix("vp_").suffix("_cover"));
            current_id = other.clone();
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

        for vehicle in &submap.vehicles {
            for vehicle_part in &vehicle.parts {
                vehicle_part
                    .info
                    .finalize(&self.vehicle_parts_info.map, "submap vehicle");
            }
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
        id: InfoId::new("human"),
        name: ItemName::from(CddaItemName::Simple(Arc::from("Human"))),
        default_faction: Arc::from("human"),
        looks_like: Some(UntypedInfoId::new("overlay_male_mutation_SKIN_TAN")),
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
