use crate::screens::inventory::components::{InventoryAction, InventoryItemRow};
use crate::screens::inventory::resource::{ITEM_TEXT_COLOR, SELECTED_ITEM_TEXT_COLOR};
use crate::screens::inventory::section::InventorySection;
use crate::screens::inventory::systems::{InventoryButton, InventorySystem};
use crate::{DebugTextShown, Fragment, ItemHandler, ItemItem, Phrase};
use bevy::ecs::entity::hash_map::EntityHashMap;
use bevy::prelude::{
    AlignItems, BackgroundColor, ChildSpawnerCommands, Entity, JustifyContent, Node, Overflow,
    Text, TextColor, Val, debug,
};
use cdda_json_files::CommonItemInfo;
use hud::{
    ButtonBuilder, Fonts, HOVERED_BUTTON_COLOR, SMALL_SPACING, SOFT_TEXT_COLOR, SelectionList,
};

struct SectionData<'r> {
    fonts: &'r Fonts,
    debug_text_shown: &'r DebugTextShown,
    inventory_system: &'r InventorySystem,
    previous_selected_item: Option<Entity>,
    section: InventorySection,
    drop_section: bool,
}

impl SectionData<'_> {
    fn add_expansion_button(&self, parent: &mut ChildSpawnerCommands, item_text_color: TextColor) {
        parent.spawn((
            Text::default(),
            item_text_color,
            self.fonts.regular(),
            Node {
                width: Val::Px(20.0),
                overflow: Overflow::clip(),
                ..Node::default()
            },
        ));
    }

    fn add_item_name(&self, parent: &mut ChildSpawnerCommands, item_phrase: &Phrase) {
        parent
            .spawn((
                Text::default(),
                SOFT_TEXT_COLOR,
                self.fonts.regular(),
                Node {
                    width: Val::Px(500.0),
                    overflow: Overflow::clip(),
                    ..Node::default()
                },
            ))
            .with_children(|parent| {
                for (span, color, debug) in item_phrase.as_text_sections() {
                    let mut entity = parent.spawn((span, color, self.fonts.regular()));
                    if let Some(debug) = debug {
                        entity
                            .insert((debug, self.debug_text_shown.text_font(self.fonts.regular())));
                    }
                }
            });
    }

    fn add_item_properties(&self, parent: &mut ChildSpawnerCommands, item_info: &CommonItemInfo) {
        let property_node = Node {
            width: Val::Px(60.0),
            overflow: Overflow::clip(),
            justify_content: JustifyContent::End,
            ..Node::default()
        };

        parent.spawn((
            Text::from(if let Some(ref volume) = item_info.volume {
                format!("{volume}")
            } else {
                String::new()
            }),
            SOFT_TEXT_COLOR,
            self.fonts.regular(),
            property_node.clone(),
        ));

        parent.spawn((
            Text::from(if let Some(ref mass) = item_info.mass {
                format!("{mass}")
            } else {
                String::new()
            }),
            SOFT_TEXT_COLOR,
            self.fonts.regular(),
            property_node,
        ));
    }

    fn add_item_action_buttons(
        &self,
        parent: &mut ChildSpawnerCommands<'_>,
        item_entity: Entity,
        item_text_color: TextColor,
    ) {
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

        for action in actions {
            let caption = format!("{}", &action);
            ButtonBuilder::new(
                caption,
                item_text_color,
                self.fonts.regular(),
                self.inventory_system.0,
            )
            .spawn(
                parent,
                InventoryButton {
                    item: item_entity,
                    action,
                },
            );
        }
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
        let is_selected;
        let is_selected_previous;
        if let Some(previous_selected_item) = section_data.previous_selected_item {
            is_selected = item_entity == previous_selected_item;
            is_selected_previous = is_selected;
        } else {
            is_selected = self.selection_list.selected.is_none();
            is_selected_previous = false;
        }

        let background_color = if is_selected {
            HOVERED_BUTTON_COLOR
        } else {
            BackgroundColor::DEFAULT
        };
        let item_text_color = if is_selected {
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
            ))
            .with_children(|parent| {
                section_data.add_expansion_button(parent, item_text_color);
                section_data.add_item_name(parent, item_phrase);
                section_data.add_item_properties(parent, item_info);
                section_data.add_item_action_buttons(parent, item_entity, item_text_color);
            })
            .id();

        self.selection_list.append(row_entity);
        if is_selected_previous {
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
