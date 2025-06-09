use crate::{
    Amount, Containable, InPocket, ItemHierarchy, ItemIntegrity, Phrase, StandardIntegrity,
};
use crate::{SealedPocket, Shared};
use bevy::prelude::{
    App, Changed, ChildOf, Children, Entity, FixedUpdate, IntoScheduleConfigs as _, Or, Plugin,
    Query, With, resource_exists, warn,
};
use cdda_json_files::{PocketInfo, PocketType};
use gameplay_cdda::Infos;
use gameplay_location::Pos;
use std::fmt::Write as _;

pub(crate) struct ItemChecksPlugin;

impl Plugin for ItemChecksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                check_item_parents,
                check_single_item.run_if(resource_exists::<Infos>),
                check_integrities,
            ),
        );
    }
}

fn check_item_parents(
    checked_item: Query<
        (
            Entity,
            Option<&Shared<PocketInfo>>,
            Option<&SealedPocket>,
            Option<&ChildOf>,
            Option<&InPocket>,
        ),
        Or<(With<Amount>, With<Containable>)>,
    >,
    pos: Query<(), With<Pos>>,
    pockets: Query<(), With<Shared<PocketInfo>>>,
) {
    if cfg!(debug_assertions) {
        assert!(
            checked_item
                .iter()
                .all(|(_, pocket_info, ..)| pocket_info.is_none()),
            "Items should not have pocket info"
        );
        assert!(
            checked_item
                .iter()
                .all(|(.., sealed_pocket, _, _)| sealed_pocket.is_none()),
            "Items should not sealed"
        );

        for (entity, .., child_of, in_pocket) in &checked_item {
            let has_pos = pos.contains(entity);
            let is_in_area = child_of.is_some();
            let is_in_pocket = in_pocket.is_some();
            let area_found_as_pocket =
                child_of.is_some_and(|child_of| pockets.contains(child_of.parent()));
            let pocket_found_as_pocket =
                in_pocket.is_some_and(|in_pocket| pockets.contains(in_pocket.pocket_entity));

            if has_pos {
                assert!(
                    is_in_area,
                    "Items with a pos ({pos:?}) should be in an area ({child_of:?}, {in_pocket:?}, {area_found_as_pocket:?}, {pocket_found_as_pocket:?})"
                );
                assert!(
                    !is_in_pocket,
                    "Items with a pos ({pos:?}) should not be in a pocket ({child_of:?}, {in_pocket:?}, {area_found_as_pocket:?}, {pocket_found_as_pocket:?})"
                );

                assert!(
                    !area_found_as_pocket,
                    "The area of an items should not be a pocket ({child_of:?}, {in_pocket:?}, {area_found_as_pocket:?}, {pocket_found_as_pocket:?})"
                );
            } else {
                assert!(
                    !is_in_area,
                    "Items without a pos ({pos:?}) should not be in an area ({child_of:?}, {in_pocket:?}, {area_found_as_pocket:?}, {pocket_found_as_pocket:?})"
                );
                assert!(
                    is_in_pocket,
                    "Items without a pos ({pos:?}) should be in a pocket ({child_of:?}, {in_pocket:?}, {area_found_as_pocket:?}, {pocket_found_as_pocket:?})"
                );

                assert!(
                    pocket_found_as_pocket,
                    "The pocket of an items should be a pocket ({child_of:?}, {in_pocket:?}, {area_found_as_pocket:?}, {pocket_found_as_pocket:?})"
                );
            }
        }
    }
}

#[expect(clippy::needless_pass_by_value)]
fn check_single_item(
    item_hierarchy: ItemHierarchy,
    pockets: Query<(Entity, &Shared<PocketInfo>), Changed<Children>>,
) {
    if cfg!(debug_assertions) {
        for (pocket_entity, pocket_info) in pockets.iter() {
            let in_pocket = InPocket { pocket_entity };
            let count = item_hierarchy.items_in_pocket(in_pocket).count();
            match pocket_info.pocket_type {
                PocketType::MagazineWell => {
                    if 1 < count {
                        warn!(
                            "At most one item expected in {pocket_info:?} ({in_pocket:?}) instead of {count}: {}",
                            item_hierarchy.items_in_pocket(in_pocket).fold(
                                String::new(),
                                |mut output, item| {
                                    write!(
                                        output,
                                        "\n- {}",
                                        Phrase::from_fragments(item.fragments().collect())
                                    )
                                    .expect("Writing should succeed");
                                    output
                                }
                            )
                        );
                    }
                }
                PocketType::Magazine => {
                    if count != 1 {
                        warn!(
                            "Exactly one item expected in {pocket_info:?} ({in_pocket:?}) instead of {count}: {}",
                            item_hierarchy.items_in_pocket(in_pocket).fold(
                                String::new(),
                                |mut output, item| {
                                    write!(
                                        output,
                                        "\n- {}",
                                        Phrase::from_fragments(item.fragments().collect())
                                    )
                                    .expect("Writing should succeed");
                                    output
                                }
                            )
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

fn check_integrities(integrities: Query<(&ItemIntegrity, &StandardIntegrity)>) {
    assert!(
        !cfg!(debug_assertions) || integrities.is_empty(),
        "ItemIntegrity and StandardIntegrity may not be combined"
    );
}
