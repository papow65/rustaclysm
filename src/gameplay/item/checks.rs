use crate::gameplay::item::Pocket;
use crate::gameplay::phrase::Phrase;
use crate::gameplay::{
    Amount, Containable, Infos, ItemHierarchy, ItemIntegrity, ObjectCategory, ObjectDefinition,
    Pos, StandardIntegrity,
};
use bevy::prelude::{
    App, Changed, Children, Entity, FixedUpdate, IntoSystemConfigs as _, Or, Parent, Plugin, Query,
    With, resource_exists,
};
use cdda_json_files::PocketType;

pub(crate) struct ItemChecksPlugin;

impl Plugin for ItemChecksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                check_item_category,
                check_item_parents,
                check_single_item.run_if(resource_exists::<Infos>),
                check_integrities,
            ),
        );
    }
}

#[expect(clippy::needless_pass_by_value)]
fn check_item_category(items: Query<&ObjectDefinition, Or<(With<Amount>, With<Containable>)>>) {
    if cfg!(debug_assertions) {
        let definition = items
            .iter()
            .find(|definition| definition.category != ObjectCategory::Item);
        assert_eq!(
            definition, None,
            "Incorrect category for item {definition:?}"
        );
    }
}

#[expect(clippy::needless_pass_by_value)]
fn check_item_parents(
    checked_item: Query<
        (Entity, Option<&Pocket>, Option<&Parent>),
        Or<(With<Amount>, With<Containable>)>,
    >,
    pos: Query<(), With<Pos>>,
    pockets: Query<(), With<Pocket>>,
) {
    if cfg!(debug_assertions) {
        assert!(
            checked_item.iter().all(|(_, pocket, _)| pocket.is_none()),
            "Items should not be pockets"
        );
        assert!(
            checked_item.iter().all(|(entity, _, parent)| parent
                .inspect(|parent| {
                    let pos = pos.contains(entity);
                    let pocket = pockets.contains(parent.get());
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
    pockets: Query<(Entity, &Pocket), Changed<Children>>,
) {
    if cfg!(debug_assertions) {
        for (entity, pocket) in pockets.iter() {
            let count = item_hierarchy.items_in(entity).count();
            match pocket.type_ {
                PocketType::MagazineWell => {
                    if 1 < count {
                        eprintln!(
                            "At most one item expected in {pocket:?} ({entity:?}) instead of {count}:"
                        );
                        for item in item_hierarchy.items_in(entity) {
                            eprintln!("- {}", Phrase::from_fragments(item.fragments().collect()));
                        }
                    }
                }
                PocketType::Magazine => {
                    if count != 1 {
                        eprintln!(
                            "Exactly one item expected in {pocket:?} ({entity:?}) instead of {count}:"
                        );
                        for item in item_hierarchy.items_in(entity) {
                            eprintln!("- {}", Phrase::from_fragments(item.fragments().collect()));
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[expect(clippy::needless_pass_by_value)]
fn check_integrities(integrities: Query<(&ItemIntegrity, &StandardIntegrity)>) {
    assert!(
        !cfg!(debug_assertions) || integrities.is_empty(),
        "ItemIntegrity and StandardIntegrity may not be combined"
    );
}
