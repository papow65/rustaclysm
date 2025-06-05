use crate::{
    Amount, Containable, Infos, ItemHierarchy, ItemIntegrity, Phrase, Pos, StandardIntegrity,
};
use crate::{SealedPocket, Shared};
use bevy::prelude::{
    App, Changed, ChildOf, Children, Entity, FixedUpdate, IntoScheduleConfigs as _, Or, Plugin,
    Query, With, resource_exists, warn,
};
use cdda_json_files::{PocketInfo, PocketType};
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
                .all(|(.., sealed_pocket, _)| sealed_pocket.is_none()),
            "Items should not sealed"
        );
        assert!(
            checked_item.iter().all(|(entity, .., child_of)| child_of
                .inspect(|child_of| {
                    let pos = pos.contains(entity);
                    let pocket = pockets.contains(child_of.parent());
                    assert_eq!(
                        pos, !pocket,
                        "Items should either have a pos ({pos:?}), xor a parent pocket ({pocket:?})"
                    );
                })
                .is_some()),
            "All items should have a parent"
        );
    }
}

#[expect(clippy::needless_pass_by_value)]
fn check_single_item(
    item_hierarchy: ItemHierarchy,
    pockets: Query<(Entity, &Shared<PocketInfo>), Changed<Children>>,
) {
    if cfg!(debug_assertions) {
        for (entity, pocket_info) in pockets.iter() {
            let count = item_hierarchy.items_in(entity).count();
            match pocket_info.pocket_type {
                PocketType::MagazineWell => {
                    if 1 < count {
                        warn!(
                            "At most one item expected in {pocket_info:?} ({entity:?}) instead of {count}: {}",
                            item_hierarchy.items_in(entity).fold(
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
                            "Exactly one item expected in {pocket_info:?} ({entity:?}) instead of {count}: {}",
                            item_hierarchy.items_in(entity).fold(
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
