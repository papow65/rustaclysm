use crate::gameplay::item::Pocket;
use crate::gameplay::{Amount, Containable, ItemIntegrity, Pos, StandardIntegrity};
use bevy::prelude::{App, Entity, FixedUpdate, Or, Parent, Plugin, Query, With};

pub(crate) struct ItemChecksPlugin;

impl Plugin for ItemChecksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (check_item_parents, check_integrity));
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
            checked_item.iter().all(|(entity, _, parent)| parent.inspect(|parent| {
                let pos = pos.contains(entity);
                let pocket = pockets.contains(parent.get());
                assert_eq!(pos, !pocket, "Items should either have a pos ({pos:?}), xor a parent pocket ({pocket:?})");
            }).is_some()),
                "All items should have a parent"
        );
    }
}

#[expect(clippy::needless_pass_by_value)]
fn check_integrity(integrities: Query<(&ItemIntegrity, &StandardIntegrity)>) {
    assert!(
        !cfg!(debug_assertions) || integrities.is_empty(),
        "ItemIntegrity and StandardIntegrity may not be combined"
    );
}
