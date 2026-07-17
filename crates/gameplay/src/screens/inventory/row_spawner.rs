use crate::screens::inventory::{
    ITEM_TEXT_COLOR, InventoryAction, InventoryButton, InventoryItemRow, InventorySection,
    InventorySystem, SELECTED_ITEM_TEXT_COLOR,
};
use bevy::ecs::entity::hash_map::EntityHashMap;
use bevy::ecs::spawn::{SpawnIter, SpawnWith};
use bevy::picking::Pickable;
use bevy::prelude::{
    AlignItems, BackgroundColor, Bundle, ChildSpawner, ChildSpawnerCommands, Children, Entity,
    JustifyContent, Node, Overflow, Spawn, SpawnRelated as _, Text, TextColor, Val, debug,
};
use cdda_json_files::CommonItemInfo;
use gameplay_item::{ItemHandler, ItemItem};
use hud::{ButtonBuilder, HOVERED_BUTTON_COLOR, SMALL_SPACING, SOFT_TEXT_COLOR};
use text::{Fragment, Phrase};
use util::Maybe;

struct SectionData<'r> {
    inventory_system: &'r InventorySystem,
    previous_selected_item: Option<Entity>,
    section: InventorySection,
    drop_section: bool,
}

impl SectionData<'_> {
    fn expansion_button(item_text_color: TextColor) -> impl Bundle {
        (
            Text::default(),
            item_text_color,
            Node {
                width: Val::Px(20.0),
                overflow: Overflow::clip(),
                ..Node::default()
            },
        )
    }

    fn item_name(item_phrase: &Phrase) -> impl Bundle {
        let text_sections = item_phrase.as_text_sections();

        (
            Text::default(),
            SOFT_TEXT_COLOR,
            Node {
                width: Val::Px(500.0),
                overflow: Overflow::clip(),
                ..Node::default()
            },
            Children::spawn((SpawnWith(move |parent: &mut ChildSpawner| {
                for (span, color, debug) in text_sections {
                    parent.spawn((span, color, Maybe(debug)));
                }
            }),)),
            Pickable::IGNORE,
        )
    }

    fn item_properties(item_info: &CommonItemInfo) -> [impl Bundle; 2] {
        let property_node = Node {
            width: Val::Px(60.0),
            overflow: Overflow::clip(),
            justify_content: JustifyContent::End,
            ..Node::default()
        };

        [
            (
                Text::from(if let Some(ref volume) = item_info.volume {
                    format!("{volume}")
                } else {
                    String::new()
                }),
                SOFT_TEXT_COLOR,
                property_node.clone(),
            ),
            (
                Text::from(if let Some(ref mass) = item_info.mass {
                    format!("{mass}")
                } else {
                    String::new()
                }),
                SOFT_TEXT_COLOR,
                property_node,
            ),
        ]
    }

    fn item_action_buttons(
        &self,
        item_entity: Entity,
        item_text_color: TextColor,
    ) -> Vec<impl Bundle> {
        let mut actions = vec![InventoryAction::Examine];
        if matches!(self.section, InventorySection::Nbor(_)) {
            actions.push(InventoryAction::Take);
            if !self.drop_section {
                actions.push(InventoryAction::Move);
            }
        } else {
            actions.push(InventoryAction::Drop);
        }
        if matches!(self.section, InventorySection::Hands) {
            actions.push(InventoryAction::Unwield);
        } else {
            actions.push(InventoryAction::Wield);
        }

        actions
            .into_iter()
            .map(move |action| {
                ButtonBuilder::new(
                    format!("{action}"),
                    item_text_color,
                    self.inventory_system.0,
                    InventoryButton {
                        item: item_entity,
                        action,
                    },
                )
                .bundle()
            })
            .collect()
    }
}

struct InventoryBuilder<'r, 'c> {
    section_by_item: &'r mut EntityHashMap<InventorySection>,
    panel: &'r mut ChildSpawnerCommands<'c>,
    selectable_items: &'r mut Vec<Entity>,
    selected_item: &'r mut Option<Entity>,
}

impl InventoryBuilder<'_, '_> {
    fn add_row(
        &mut self,
        section_data: &SectionData,
        item_entity: Entity,
        item_phrase: &Phrase,
        item_info: &CommonItemInfo,
    ) {
        #[derive(PartialEq)]
        enum Selection {
            PreviousSelected,
        }

        let selecttion = section_data
            .previous_selected_item
            .and_then(|previous_selected_item| {
                (item_entity == previous_selected_item).then_some(Selection::PreviousSelected)
            });

        let background_color = if selecttion.is_some() {
            HOVERED_BUTTON_COLOR
        } else {
            BackgroundColor::DEFAULT
        };
        let item_text_color = if selecttion.is_some() {
            SELECTED_ITEM_TEXT_COLOR
        } else {
            ITEM_TEXT_COLOR
        };

        /*let identation = Fragment::colorized(
             *       if let Some(in_pocket) = in_pocket {
             *           format!("{}'->", "    ".repeat(in_pocket.depth.get() - 1))
        } else {
            String::new()
        },
        SOFT_TEXT_COLOR,
        );*/

        let row = self
            .panel
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    column_gap: SMALL_SPACING,
                    ..Node::default()
                },
                InventoryItemRow { item: item_entity },
                background_color,
                Pickable::IGNORE,
                Children::spawn((
                    Spawn(SectionData::expansion_button(item_text_color)),
                    Spawn(SectionData::item_name(item_phrase)),
                    SpawnIter(SectionData::item_properties(item_info).into_iter()),
                    SpawnIter(
                        section_data
                            .item_action_buttons(item_entity, item_text_color)
                            .into_iter(),
                    ),
                )),
            ))
            .id();
        self.selectable_items.push(row);

        if selecttion == Some(Selection::PreviousSelected) {
            debug!("Previous selected found");
            *self.selected_item = Some(row);
        }
        self.section_by_item
            .insert(item_entity, section_data.section);
    }
}

pub(super) struct RowSpawner<'r, 'c> {
    builder: InventoryBuilder<'r, 'c>,
    section_data: SectionData<'r>,
}

impl<'r, 'c> RowSpawner<'r, 'c>
where
    'c: 'r,
{
    pub(super) const fn new(
        inventory_system: &'r InventorySystem,
        section_by_item: &'r mut EntityHashMap<InventorySection>,
        inventory_panel: &'r mut ChildSpawnerCommands<'c>,
        previous_selected_item: Option<Entity>,
        section: InventorySection,
        drop_section: bool,
        selectable_items: &'r mut Vec<Entity>,
        selected_item: &'r mut Option<Entity>,
    ) -> Self {
        Self {
            builder: InventoryBuilder {
                section_by_item,
                panel: inventory_panel,
                selectable_items,
                selected_item,
            },
            section_data: SectionData {
                inventory_system,
                previous_selected_item,
                section,
                drop_section,
            },
        }
    }
}

impl ItemHandler for RowSpawner<'_, '_> {
    fn handle_item(&mut self, item: &ItemItem, item_fragments: Vec<Fragment>) {
        self.builder.add_row(
            &self.section_data,
            item.entity,
            &Phrase::from_fragments(item_fragments),
            item.common_info,
        );
    }

    fn show_other_pockets(&self) -> bool {
        false
    }
}
