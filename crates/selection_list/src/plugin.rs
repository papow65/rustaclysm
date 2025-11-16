use crate::systems::create_selection_list_key_bindings;
use bevy::ecs::query::QueryFilter;
use bevy::prelude::{App, Entity, In, IntoSystem, OnEnter, States};

pub fn selection_list_plugin<
    S: States + Default,
    Marker: 'static,
    I: IntoSystem<In<(Option<Entity>, Option<Entity>)>, (), Marker> + Clone + Sync + Send + 'static,
    Q: QueryFilter + 'static,
>(
    app: &mut App,
    state: S,
    select_element_label: &'static str,
    adapt_to_selection: I,
) {
    app.add_systems(
        OnEnter(state.clone()),
        create_selection_list_key_bindings::<S, Marker, I, Q>(
            state,
            select_element_label,
            adapt_to_selection,
        ),
    );
}
