use crate::screens::inventory::{
    ITEM_TEXT_COLOR, InventoryAction, InventoryButton, InventoryItemRow, InventorySection,
    InventorySystem, SELECTED_ITEM_TEXT_COLOR,
};
use crate::{DebugTextShown, Fragment, ItemHandler, ItemItem, Phrase};
use bevy::ecs::entity::hash_map::EntityHashMap;
use bevy::ecs::spawn::{SpawnIter, SpawnWith};
use bevy::picking::Pickable;
use bevy::prelude::{
    AlignItems, BackgroundColor, Bundle, ChildSpawner, ChildSpawnerCommands, Children, Entity,
    JustifyContent, Node, Overflow, Spawn, SpawnRelated as _, Text, TextColor, Val, debug,
};
use cdda_json_files::CommonItemInfo;
use hud::{ButtonBuilder, Fonts, HOVERED_BUTTON_COLOR, SMALL_SPACING, SOFT_TEXT_COLOR};
use selection_list::SelectionList;

struct SectionData<'r> {
    fonts: &'r Fonts,
    debug_text_shown: &'r DebugTextShown,
    inventory_system: &'r InventorySystem,
    previous_selected_item: Option<Entity>,
    section: InventorySection,
    drop_section: bool,
}

impl SectionData<'_> {
    fn expansion_button(&self, item_text_color: TextColor) -> impl Bundle {
        (
            Text::default(),
            item_text_color,
            self.fonts.regular(),
            Node {
                width: Val::Px(20.0),
                overflow: Overflow::clip(),
                ..Node::default()
            },
            Pickable::IGNORE,
        )
    }

    fn item_name(&self, item_phrase: &Phrase) -> impl Bundle {
        let text_sections = item_phrase.as_text_sections();
        let regular = self.fonts.regular();
        let debug_regular = self.debug_text_shown.text_font(self.fonts.regular());

        (
            Text::default(),
            SOFT_TEXT_COLOR,
            self.fonts.regular(),
            Node {
                width: Val::Px(500.0),
                overflow: Overflow::clip(),
                ..Node::default()
            },
            Children::spawn((SpawnWith(move |parent: &mut ChildSpawner| {
                for (span, color, debug) in text_sections {
                    let mut entity = parent.spawn((span, color, regular.clone()));
                    if let Some(debug) = debug {
                        entity.insert((debug, debug_regular.clone()));
                    }
                }
            }),)),
            Pickable::IGNORE,
        )
    }

    fn item_properties(&self, item_info: &CommonItemInfo) -> [impl Bundle; 2] {
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
                self.fonts.regular(),
                Pickable::IGNORE,
                property_node.clone(),
            ),
            (
                Text::from(if let Some(ref mass) = item_info.mass {
                    format!("{mass}")
                } else {
                    String::new()
                }),
                SOFT_TEXT_COLOR,
                self.fonts.regular(),
                Pickable::IGNORE,
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
                    format!("{}", &action),
                    item_text_color,
                    self.fonts.regular(),
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
    selection_list: &'r mut SelectionList,
    section_by_item: &'r mut EntityHashMap<InventorySection>,
    panel: &'r mut ChildSpawnerCommands<'c>,
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
            FirstItem,
            PreviousSelected,
        }

        let selecttion = if let Some(previous_selected_item) = section_data.previous_selected_item {
            (item_entity == previous_selected_item).then_some(Selection::PreviousSelected)
        } else {
            (self.selection_list.selected.is_none()).then_some(Selection::FirstItem)
        };

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

        let row_entity = self
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
                    Spawn(section_data.expansion_button(item_text_color)),
                    Spawn(section_data.item_name(item_phrase)),
                    SpawnIter(section_data.item_properties(item_info).into_iter()),
                    SpawnIter(
                        section_data
                            .item_action_buttons(item_entity, item_text_color)
                            .into_iter(),
                    ),
                )),
            ))
            .id();

        self.selection_list.append(row_entity);
        if selecttion == Some(Selection::PreviousSelected) {
            debug!("Previous selected found");
            self.selection_list.selected = Some(row_entity);
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
        fonts: &'r Fonts,
        debug_text_shown: &'r DebugTextShown,
        inventory_system: &'r InventorySystem,
        selection_list: &'r mut SelectionList,
        section_by_item: &'r mut EntityHashMap<InventorySection>,
        inventory_panel: &'r mut ChildSpawnerCommands<'c>,
        previous_selected_item: Option<Entity>,
        section: InventorySection,
        drop_section: bool,
    ) -> Self {
        Self {
            builder: InventoryBuilder {
                selection_list,
                section_by_item,
                panel: inventory_panel,
            },
            section_data: SectionData {
                fonts,
                debug_text_shown,
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
