use crate::{SelectedItemIn, SelectedItemOf, SelectionListItems, SelectionListStep};
use bevy::ecs::query::QueryFilter;
use bevy::prelude::{
    Changed, Commands, ComputedNode, DespawnOnExit, Entity, In, IntoSystem as _, Local, Query,
    ScrollPosition, Single, States, UiGlobalTransform, Without, World,
};
use hud::max_scroll;
use keyboard::{Held, KeyBindings};
use manual::ManualSection;
use std::time::Instant;
use strum::VariantArray as _;
use util::log_if_slow;

pub(super) fn create_selection_list_key_bindings<S: States + Default, Q: QueryFilter + 'static>(
    state: S,
    select_element_label: &'static str,
) -> impl Fn(&mut World, Local<KeyBindings<S, (), Held>>) {
    move |world: &mut World, held_bindings: Local<KeyBindings<S, (), Held>>| {
        let start = Instant::now();

        held_bindings.spawn(world, state.clone(), |bindings| {
            for &step in SelectionListStep::VARIANTS {
                bindings.add(step, (move || step).pipe(adjust_selection::<Q>));
            }
        });

        world.spawn((
            ManualSection::new(
                &[
                    (select_element_label, "arrow up/down"),
                    (select_element_label, "page up/down"),
                ],
                99,
            ),
            DespawnOnExit(state.clone()),
        ));

        log_if_slow("create_crafting_key_bindings", start);
    }
}

#[expect(clippy::needless_pass_by_value)]
fn adjust_selection<Q: QueryFilter + 'static>(
    In(step): In<SelectionListStep>,
    mut commands: Commands,
    selection_list: Single<(Entity, &SelectionListItems, &SelectedItemOf), Q>,
) {
    let start = Instant::now();

    //warn!("Adjust selection: {step:?}");

    let (selection_list_entity, selection_list_items, previous_selected_item_of) = *selection_list;

    let previous_selected = previous_selected_item_of.selected();
    let new_selected = selection_list_items.offset(previous_selected, step);

    commands
        .entity(selection_list_entity)
        .add_one_related::<SelectedItemIn>(new_selected);

    log_if_slow("move_crafting_selection", start);
}

pub(crate) fn scroll_to_selection<Q: QueryFilter + 'static>(
    mut selection_list: Single<
        (
            &UiGlobalTransform,
            &ComputedNode,
            &mut ScrollPosition,
            &SelectedItemOf,
        ),
        (Q, Changed<SelectedItemOf>),
    >,
    ui_global_transforms: Query<&UiGlobalTransform>,
) {
    // The middle of the list is also the middle of the screen. This makes using `UiGlobalTransform` convenient.

    let (parent_transform, selection_node, ref mut scroll_position, selected_item_of) =
        *selection_list;

    let selected_entity = selected_item_of.selected();
    let transform = ui_global_transforms
        .get(selected_entity)
        .expect("Selected item should have a global transform");

    let max_scroll = max_scroll(selection_node);
    let adjustment = (transform.translation.y - parent_transform.translation.y)
        * selection_node.inverse_scale_factor;

    // let old_scroll_pos_y = scroll_position.y;
    scroll_position.y = (scroll_position.y + adjustment).clamp(0.0, max_scroll.y);
    //warn!(
    //    "scroll to selection: t.t.y {}, pt.t.y {}, sn.isf {} => adj {}, [0 ~ ms.y {}] => sp.y {}->{}",
    //    transform.translation.y,
    //    parent_transform.translation.y,
    //    selection_node.inverse_scale_factor,
    //    adjustment,
    //    max_scroll.y,
    //    old_scroll_pos_y,
    //    scroll_position.y
    //);
}

pub(crate) fn select_first_when_empty(
    mut commands: Commands,
    selection_lists: Query<
        (Entity, &SelectionListItems),
        (Changed<SelectionListItems>, Without<SelectedItemOf>),
    >,
) {
    for (selection_list_entity, selection_list_items) in &selection_lists {
        //warn!("select_first_when_empty: {:?}", selection_list_items);
        if let Some(first) = selection_list_items.first() {
            commands
                .entity(selection_list_entity)
                .add_one_related::<SelectedItemIn>(first);
        }
    }
}
