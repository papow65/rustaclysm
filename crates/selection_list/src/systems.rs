use crate::{SelectionList, SelectionListStep};
use bevy::ecs::query::QueryFilter;
use bevy::prelude::{
    ComputedNode, DespawnOnExit, Entity, In, IntoSystem, Local, Query, Res, ScrollPosition, States,
    UiGlobalTransform, UiScale, World,
};
use hud::max_scroll;
use keyboard::{Held, KeyBindings};
use manual::ManualSection;
use std::time::Instant;
use strum::VariantArray as _;
use util::log_if_slow;

pub(super) fn create_selection_list_key_bindings<
    S: States + Default,
    Marker: 'static,
    I: IntoSystem<In<(Option<Entity>, Option<Entity>)>, (), Marker> + Clone + Sync + Send + 'static,
    Q: QueryFilter + 'static,
>(
    state: S,
    select_element_label: &'static str,
    adapt_to_on_selection: I,
) -> impl Fn(&mut World, Local<KeyBindings<S, (), Held>>) {
    move |world: &mut World, held_bindings: Local<KeyBindings<S, (), Held>>| {
        let start = Instant::now();

        let selection_list_entity = world
            .query_filtered::<Entity, Q>()
            .single(world)
            .expect("Selection list should have been found");

        held_bindings.spawn(world, state.clone(), |bindings| {
            for &step in SelectionListStep::VARIANTS {
                bindings.add(
                    step,
                    (move || (selection_list_entity, step))
                        .pipe(adjust_selection)
                        .pipe(scroll_to_selection)
                        .pipe(adapt_to_on_selection.clone()),
                );
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

fn adjust_selection(
    In((selection_list_entity, step)): In<(Entity, SelectionListStep)>,
    mut selection_lists: Query<&mut SelectionList>,
) -> (Entity, Option<Entity>, Option<Entity>) {
    let start = Instant::now();

    let mut selection_list = selection_lists
        .get_mut(selection_list_entity)
        .expect("Recipe selection list should be found");

    selection_list.adjust(step);

    log_if_slow("move_crafting_selection", start);

    (
        selection_list_entity,
        selection_list.previous_selected,
        selection_list.selected,
    )
}

#[expect(clippy::needless_pass_by_value)]
fn scroll_to_selection(
    In((selection_list_entity, previous_selected, selected)): In<(
        Entity,
        Option<Entity>,
        Option<Entity>,
    )>,
    ui_scale: Res<UiScale>,
    mut nodes: Query<(&UiGlobalTransform, &ComputedNode, &mut ScrollPosition)>,
) -> (Option<Entity>, Option<Entity>) {
    // The middle of the list is also the middle of the screen. This makes using `GlobalTransform` convenient.

    if let Some(selected) = selected {
        let (&transform, ..) = nodes
            .get(selected)
            .expect("Selected item should have a global transform");

        let (parent_transform, selection_node, mut scroll_position) = nodes
            .get_mut(selection_list_entity)
            .expect("Selection list entity should be found");
        let max_scroll = max_scroll(selection_node);
        scroll_position.y = (scroll_position.y
            + (transform.translation.y - parent_transform.translation.y) / ui_scale.0)
            .clamp(0.0, max_scroll.y);
    }

    (previous_selected, selected)
}
