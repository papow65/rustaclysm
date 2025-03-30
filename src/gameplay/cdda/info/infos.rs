use crate::gameplay::cdda::info::info_map::{InfoMap, ItemInfoMapLoader};
use crate::gameplay::cdda::info::parsed_json::ParsedJson;
use crate::gameplay::{ObjectCategory, TypeId};
use bevy::prelude::{Resource, debug, error, info};
use cdda_json_files::{
    Ammo, BionicItem, Book, CddaItem, CharacterInfo, Clothing, Comestible, CommonItemInfo, Engine,
    FieldInfo, FurnitureInfo, GenericItem, Gun, Gunmod, ItemAction, ItemGroup, Link as _, Magazine,
    Overmap, OvermapTerrainInfo, PetArmor, Practice, Quality, Recipe, RequiredLinkedLater,
    Requirement, Submap, TerrainInfo, Tool, ToolClothing, Toolmod, UntypedInfoId, VehiclePartInfo,
    VehiclePartMigration, Wheel,
};
use std::{env, process::exit, time::Instant};
use util::AsyncNew;

#[derive(Resource)]
pub(crate) struct Infos {
    #[expect(unused)]
    ammos: InfoMap<Ammo>,

    #[expect(unused)]
    bionic_items: InfoMap<BionicItem>,

    #[expect(unused)]
    books: InfoMap<Book>,

    pub(crate) characters: InfoMap<CharacterInfo>,

    #[expect(unused)]
    clothings: InfoMap<Clothing>,

    #[expect(unused)]
    comestibles: InfoMap<Comestible>,

    common_item_infos: InfoMap<CommonItemInfo>,

    #[expect(unused)]
    engines: InfoMap<Engine>,

    fields: InfoMap<FieldInfo>,
    furniture: InfoMap<FurnitureInfo>,

    #[expect(unused)]
    genenric_items: InfoMap<GenericItem>,

    #[expect(unused)]
    guns: InfoMap<Gun>,

    #[expect(unused)]
    gunmods: InfoMap<Gunmod>,

    pub(crate) item_actions: InfoMap<ItemAction>,

    item_groups: InfoMap<ItemGroup>,

    pub(crate) magazines: InfoMap<Magazine>,

    #[expect(unused)]
    pet_armors: InfoMap<PetArmor>,

    #[expect(unused)]
    practices: InfoMap<Practice>,

    qualities: InfoMap<Quality>,

    pub(crate) recipes: InfoMap<Recipe>,

    requirements: InfoMap<Requirement>,

    terrain: InfoMap<TerrainInfo>,

    #[expect(unused)]
    tools: InfoMap<Tool>,

    #[expect(unused)]
    tool_clothings: InfoMap<ToolClothing>,

    #[expect(unused)]
    toolmods: InfoMap<Toolmod>,

    vehicle_parts: InfoMap<VehiclePartInfo>,

    #[expect(unused)]
    wheels: InfoMap<Wheel>,

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

        let item_migrations = InfoMap::new(&mut enriched_json_infos, TypeId::ItemMigration);
        let mut common_item_infos = InfoMap::default();
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
        //debug!("Collected {} common items", common_item_infos.len());

        let vehicle_part_migrations = InfoMap::<VehiclePartMigration>::new(
            &mut enriched_json_infos,
            TypeId::VehiclePartMigration,
        );

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
            furniture: InfoMap::new(&mut enriched_json_infos, TypeId::Furniture),
            genenric_items,
            guns,
            gunmods,
            item_actions: InfoMap::new(&mut enriched_json_infos, TypeId::ItemAction),
            item_groups: InfoMap::new(&mut enriched_json_infos, TypeId::ItemGroup),
            magazines,
            pet_armors,
            practices: InfoMap::new(&mut enriched_json_infos, TypeId::Practice),
            qualities: InfoMap::new(&mut enriched_json_infos, TypeId::ToolQuality),
            recipes: InfoMap::new(&mut enriched_json_infos, TypeId::Recipe),
            requirements: InfoMap::new(&mut enriched_json_infos, TypeId::Requirement),
            terrain: InfoMap::new(&mut enriched_json_infos, TypeId::Terrain),
            tools,
            tool_clothings,
            toolmods,
            vehicle_parts: InfoMap::new(&mut enriched_json_infos, TypeId::VehiclePart),
            wheels,
            zone_levels: InfoMap::new(&mut enriched_json_infos, TypeId::OvermapTerrain),
        }
        .link_all(&vehicle_part_migrations);

        let mut missing_types = enriched_json_infos
            .into_keys()
            .map(|type_id| format!("{type_id:?}",))
            .collect::<Vec<_>>();
        if !missing_types.is_empty() {
            missing_types.sort();
            error!(
                "{} unprocessed types: {}",
                missing_types.len(),
                missing_types.join(", ")
            );
        }

        this.characters.add_default_human();

        let duration = start.elapsed();
        info!("The creation of Infos took {duration:?}");

        if env::var("EXIT_AFTER_INFOS") == Ok(String::from("1")) {
            exit(0);
        }

        this
    }

    fn link_all(mut self, vehicle_part_migrations: &InfoMap<VehiclePartMigration>) -> Self {
        self.common_item_infos.link_common_items(&self.qualities);
        self.furniture
            .link_furniture(&self.common_item_infos, &self.item_groups);
        self.qualities.link_qualities(&self.item_actions);
        self.requirements
            .link_requirements(&self.qualities, &self.common_item_infos);
        self.recipes
            .link_recipes(&self.qualities, &self.requirements, &self.common_item_infos);
        self.terrain.fix_and_link_terrain(
            &self.furniture,
            &self.common_item_infos,
            &self.item_groups,
        );
        self.vehicle_parts.add_wiring();
        self.vehicle_parts.link_items(&self.common_item_infos);
        self.vehicle_parts
            .add_vehicle_part_migrations(vehicle_part_migrations.values());

        self
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
                .vehicle_parts
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

    pub(crate) fn link_overmap(&self, overmap: &Overmap) {
        if overmap.linked.set(()).is_err() {
            return;
        }

        for (_, monster) in &overmap.monster_map.0 {
            monster.info.finalize(&self.characters, "overmap monster");
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
                .finalize(&self.terrain, "submap terrain");
        }
        for furniture_at in &submap.furniture {
            furniture_at
                .obj
                .finalize(&self.furniture, "submap furniture");
        }

        for fields_at in &submap.fields.0 {
            for field in &fields_at.obj.0 {
                field.field_info.finalize(&self.fields, "submap field");
            }
        }

        for character in &submap.spawns {
            self.link_character(&character.info, "submap spawn");
        }

        for vehicle in &submap.vehicles {
            for vehicle_part in &vehicle.parts {
                vehicle_part
                    .info
                    .finalize(&self.vehicle_parts, "submap vehicle");
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
            .finalize(&self.common_item_infos, "submap item");
        item.corpse.finalize(&self.characters, "submap item corpse");

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
        character.finalize(&self.characters, err_description);
    }
}

impl AsyncNew<Self> for Infos {
    async fn async_new() -> Self {
        Self::load()
    }
}
