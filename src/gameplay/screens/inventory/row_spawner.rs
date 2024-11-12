use crate::gameplay::item::{InPocket, Subitems};
use crate::gameplay::screens::inventory::components::{InventoryAction, InventoryItemRow};
use crate::gameplay::screens::inventory::resource::{ITEM_TEXT_COLOR, SELECTED_ITEM_TEXT_COLOR};
use crate::gameplay::screens::inventory::section::InventorySection;
use crate::gameplay::screens::inventory::systems::{InventoryButton, InventorySystem};
use crate::gameplay::{DebugTextShown, Fragment, Infos, ItemHierarchyWalker, ItemItem, Phrase};
use crate::hud::{
    ButtonBuilder, Fonts, SelectionList, HOVERED_BUTTON_COLOR, SMALL_SPACING, SOFT_TEXT_COLOR,
};
use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::{
    AlignItems, BackgroundColor, BuildChildren, ChildBuild, ChildBuilder, Entity, JustifyContent,
    Node, Overflow, Text, TextColor, Val,
};
use cdda_json_files::{CommonItemInfo, PocketType};
use std::{cell::RefCell, iter::once};

struct InventoryBuilder<'r, 'c> {
    selection_list: &'r mut SelectionList,
    section_by_item: &'r mut EntityHashMap<InventorySection>,
    panel: &'r mut ChildBuilder<'c>,
}

pub(super) struct RowSpawner<'r, 'c> {
    fonts: &'r Fonts,
    infos: &'r Infos,
    debug_text_shown: &'r DebugTextShown,
    inventory_system: &'r InventorySystem,
    builder: RefCell<InventoryBuilder<'r, 'c>>,
    previous_selected_item: Option<Entity>,
    section: InventorySection,
    drop_section: bool,
}

impl<'r, 'c> RowSpawner<'r, 'c>
where
    'c: 'r,
{
    pub(super) fn new(
        fonts: &'r Fonts,
        infos: &'r Infos,
        debug_text_shown: &'r DebugTextShown,
        inventory_system: &'r InventorySystem,
        selection_list: &'r mut SelectionList,
        section_by_item: &'r mut EntityHashMap<InventorySection>,
        inventory_panel: &'r mut ChildBuilder<'c>,
        previous_selected_item: Option<Entity>,
        section: InventorySection,
        drop_section: bool,
    ) -> Self {
        Self {
            fonts,
            infos,
            debug_text_shown,
            inventory_system,
            builder: RefCell::new(InventoryBuilder {
                selection_list,
                section_by_item,
                panel: inventory_panel,
            }),
            previous_selected_item,
            section,
            drop_section,
        }
    }

    fn add_row<'p>(
        &self,
        in_pocket: Option<InPocket>,
        item: &ItemItem,
        item_info: &CommonItemInfo,
        magazines: impl Iterator<Item = Subitems<'p>>,
    ) {
        let mut builder = self.builder.borrow_mut();

        let is_selected;
        let is_selected_previous;
        if let Some(previous_selected_item) = self.previous_selected_item {
            is_selected = item.entity == previous_selected_item;
            is_selected_previous = is_selected;
        } else {
            is_selected = builder.selection_list.selected.is_none();
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

        let identation = Fragment::colorized(
            if let Some(in_pocket) = in_pocket {
                format!("{}'->", "    ".repeat(in_pocket.depth.get() - 1))
            } else {
                String::new()
            },
            SOFT_TEXT_COLOR,
        );

        let row_entity = builder
            .panel
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    column_gap: SMALL_SPACING,
                    ..Node::default()
                },
                InventoryItemRow { item: item.entity },
                background_color,
            ))
            .with_children(|parent| {
                self.add_expansion_button(parent, item_text_color);
                self.add_item_name(
                    parent,
                    &Phrase::from_fragment(identation)
                        .extend(item.fragments())
                        .debug(format!("[{}]", item.definition.id.fallback_name()))
                        .extend(magazines.flat_map(|info| info.output)),
                );
                self.add_item_properties(parent, item_info);
                self.add_item_action_buttons(parent, item.entity, item_text_color);
            })
            .id();

        builder.selection_list.append(row_entity);
        if is_selected_previous {
            println!("Previous selected found");
            builder.selection_list.selected = Some(row_entity);
        }
        builder.section_by_item.insert(item.entity, self.section);
    }

    fn add_expansion_button(&self, parent: &mut ChildBuilder, item_text_color: TextColor) {
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

    fn add_item_name(&self, parent: &mut ChildBuilder, item_phrase: &Phrase) {
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

    fn add_item_properties(&self, parent: &mut ChildBuilder, item_info: &CommonItemInfo) {
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
        parent: &mut ChildBuilder<'_>,
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

impl<'r, 'c> ItemHierarchyWalker for RowSpawner<'r, 'c> {
    fn visit_item<'p>(
        &'p self,
        item: ItemItem,
        contents: impl Iterator<Item = Subitems<'p>>,
        magazines: impl Iterator<Item = Subitems<'p>>,
        magazine_wells: impl Iterator<Item = Subitems<'p>>,
        _other_pockets: impl Iterator<Item = Subitems<'p>>,
        in_pocket: Option<InPocket>,
    ) -> Vec<Fragment> {
        let Some(item_info) = self.infos.try_common_item_info(&item.definition.id) else {
            eprintln!("Unknown item: {:?}", item.definition.id);
            return Vec::new();
        };

        match in_pocket {
            None
            | Some(InPocket {
                type_: PocketType::Container | PocketType::MagazineWell,
                ..
            }) => {
                self.add_row(in_pocket, &item, item_info, magazines);
                // Iterating magazine wells and contents is required.
                _ = magazine_wells.chain(contents).collect::<Vec<_>>();
                Vec::new()
            }
            Some(InPocket {
                type_: PocketType::Magazine,
                ..
            }) => once(Fragment::colorized("with", SOFT_TEXT_COLOR))
                .chain(item.fragments())
                .collect(),
            _ => Vec::new(),
        }
    }
}
